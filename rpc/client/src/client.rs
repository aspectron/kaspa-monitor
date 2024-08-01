use crate::error::Error;
use crate::imports::*;
pub use kaspa_monitor_macros::build_wrpc_client_interface;
use kaspa_monitor_rpc_core::prelude::*;
use std::fmt::Debug;
use workflow_core::channel::Multiplexer;
use workflow_rpc::client::Ctl as WrpcCtl;
pub use workflow_rpc::client::{
    result::Result as ClientResult, ConnectOptions, ConnectResult, ConnectStrategy,
    Resolver as RpcResolver, ResolverResult, WebSocketConfig, WebSocketError,
};
pub use workflow_rpc::encoding::Encoding as WrpcEncoding;
use workflow_serializer::prelude::Serializable;

struct Inner {
    rpc_client: Arc<RpcClient<RpcApiOps>>,
    notification_relay_channel: Channel<Notification>,
    notification_intake_channel: Mutex<Channel<Notification>>,
    encoding: Encoding,
    wrpc_ctl_multiplexer: Multiplexer<WrpcCtl>,
    background_services_running: Arc<AtomicBool>,
    service_ctl: DuplexChannel<()>,
    connect_guard: AsyncMutex<()>,
    disconnect_guard: AsyncMutex<()>,
}

impl Inner {
    pub fn try_new(url: Option<&str>, encoding: Encoding) -> Result<Inner> {
        let ctl_multiplexer = Multiplexer::<WrpcCtl>::new();
        let options = RpcClientOptions {
            url,
            ctl_multiplexer: Some(ctl_multiplexer.clone()),
        };

        let notification_relay_channel = Channel::unbounded();
        let notification_intake_channel = Mutex::new(Channel::unbounded());

        let mut interface = Interface::<RpcApiOps>::new();

        [RpcApiOps::Notify].into_iter().for_each(|notification_op| {
            // TODO - replace with Multiplexer
            let notification_sender_ = notification_relay_channel.sender.clone();
            interface.notification(
                notification_op,
                workflow_rpc::client::Notification::new(
                    move |notification: Serializable<Notification>| {
                        // workflow_rpc::client::Notification::new(move |notification: Notification| {
                        let notification_sender = notification_sender_.clone();
                        Box::pin(async move {
                            // println!("Receiving notification: {:?}", notification);

                            // log_info!("notification receivers: {}", notification_sender.receiver_count());
                            // log_trace!("notification {:?}", notification);
                            if notification_sender.receiver_count() > 1 {
                                // log_info!("notification: posting to channel: {notification:?}");
                                notification_sender.send(notification.into_inner()).await?;
                            } else {
                                log_warn!(
                                    "WARNING: wRPC notification is not consumed by client: {:?}",
                                    notification.into_inner()
                                );
                            }
                            Ok(())
                        })
                    },
                ),
            );
        });

        let rpc = Arc::new(RpcClient::new_with_encoding(
            encoding,
            interface.into(),
            options,
            None,
        )?);
        let client = Self {
            rpc_client: rpc,
            notification_relay_channel,
            notification_intake_channel,
            encoding,
            wrpc_ctl_multiplexer: ctl_multiplexer,
            service_ctl: DuplexChannel::unbounded(),
            background_services_running: Arc::new(AtomicBool::new(false)),
            connect_guard: async_std::sync::Mutex::new(()),
            disconnect_guard: async_std::sync::Mutex::new(()),
        };
        Ok(client)
    }
}

impl Debug for Inner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SparkleRpcClient")
            .field("rpc", &"rpc")
            .field("encoding", &self.encoding)
            .finish()
    }
}

#[derive(Clone)]
pub struct MonitorRpcClient {
    inner: Arc<Inner>,
}

impl Debug for MonitorRpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SparkleRpcClient")
            .field("url", &self.url())
            .field("connected", &self.is_connected())
            .finish()
    }
}

impl Default for MonitorRpcClient {
    fn default() -> Self {
        Self::try_new(None, None).unwrap()
    }
}

impl MonitorRpcClient {
    pub fn try_new(url: Option<&str>, encoding: Option<Encoding>) -> Result<MonitorRpcClient> {
        let inner = Arc::new(Inner::try_new(url, encoding.unwrap_or(Encoding::Borsh))?);
        let client = MonitorRpcClient { inner };
        Ok(client)
    }

    pub fn url(&self) -> Option<String> {
        self.inner.rpc_client.url()
    }

    pub fn is_connected(&self) -> bool {
        self.inner.rpc_client.is_connected()
    }

    pub fn encoding(&self) -> Encoding {
        self.inner.encoding
    }

    pub fn rpc_client(&self) -> &Arc<RpcClient<RpcApiOps>> {
        &self.inner.rpc_client
    }

    pub fn ctl_multiplexer(&self) -> Multiplexer<WrpcCtl> {
        self.inner.wrpc_ctl_multiplexer.clone()
    }

    // pub fn rpc_api(self: &Arc<Self>) -> Arc<dyn RpcApi> {
    //     self.clone()
    // }

    /// Start background RPC services.
    pub async fn start(&self) -> Result<()> {
        if !self
            .inner
            .background_services_running
            .load(Ordering::SeqCst)
        {
            self.inner
                .background_services_running
                .store(true, Ordering::SeqCst);
            self.start_rpc_service().await?;
        }

        Ok(())
    }

    /// Stop background RPC services.
    pub async fn stop(&self) -> Result<()> {
        if self
            .inner
            .background_services_running
            .load(Ordering::SeqCst)
        {
            self.stop_rpc_service().await?;
            self.inner
                .background_services_running
                .store(false, Ordering::SeqCst);
        }

        Ok(())
    }

    pub async fn connect(&self, options: Option<ConnectOptions>) -> ConnectResult<Error> {
        let _guard = self.inner.connect_guard.lock().await;

        let options = options.unwrap_or_default();
        let strategy = options.strategy;

        // 1Gb message and frame size limits (on native and NodeJs platforms)
        let ws_config = WebSocketConfig {
            max_message_size: Some(1024 * 1024 * 1024),
            max_frame_size: Some(1024 * 1024 * 1024),
            accept_unmasked_frames: false,
            ..Default::default()
        };

        self.start().await?;
        self.inner.rpc_client.configure(ws_config);
        match self.inner.rpc_client.connect(options).await {
            Ok(v) => Ok(v),
            Err(err) => {
                if strategy == ConnectStrategy::Fallback {
                    let _guard = self.inner.disconnect_guard.lock().await;
                    self.inner.rpc_client.shutdown().await?;
                    self.stop().await?;
                }
                Err(err.into())
            }
        }
    }

    pub async fn disconnect(&self) -> Result<()> {
        let _guard = self.inner.disconnect_guard.lock().await;

        self.inner.rpc_client.shutdown().await?;
        self.stop().await?;
        Ok(())
    }

    pub fn connect_as_task(&self) -> Result<()> {
        let self_ = self.clone();
        workflow_core::task::spawn(async move {
            log_info!("wRPC Calling connect fn...");
            self_
                .inner
                .rpc_client
                .connect(ConnectOptions::default())
                .await
                .ok();
            log_info!("wRPC Connect fn returned...");
        });
        Ok(())
    }

    pub fn notification_channel_receiver(&self) -> Receiver<Notification> {
        self.inner
            .notification_intake_channel
            .lock()
            .unwrap()
            .receiver
            .clone()
    }

    async fn start_rpc_service(&self) -> Result<()> {
        let inner = self.inner.clone();
        let wrpc_ctl_channel = inner.wrpc_ctl_multiplexer.channel();
        let notification_relay_channel = inner.notification_relay_channel.clone();
        spawn(async move {
            loop {
                select! {
                    _ = inner.service_ctl.request.receiver.recv().fuse() => {
                        break;
                    },
                    msg = notification_relay_channel.receiver.recv().fuse() => {
                        if let Ok(msg) = msg {
                            if let Err(err) = inner.notification_intake_channel.lock().unwrap().sender.try_send(msg) {
                                log_error!("notification_intake_channel.sender.try_send() error: {err}");
                            }
                        } else {
                            log_error!("notification_relay_channel receiver error");
                        }
                    }
                    msg = wrpc_ctl_channel.receiver.recv().fuse() => {
                        if let Ok(msg) = msg {
                            match msg {
                                WrpcCtl::Connect => {
                                    log_trace!("wRPC connected to {}", inner.rpc_client.url().unwrap_or("N/A".to_string()));
                                    // TODO - keep / remove?
                                }
                                WrpcCtl::Disconnect => {
                                    log_trace!("wRPC disconnected from {}", inner.rpc_client.url().unwrap_or("N/A".to_string()));
                                    // TODO - keep / remove?
                                }
                            }
                        } else {
                            log_error!("wrpc_ctl_channel.receiver.recv() error");
                        }
                    }
                }
            }
            inner.service_ctl.response.send(()).await.unwrap();
        });

        Ok(())
    }

    async fn stop_rpc_service(&self) -> Result<()> {
        self.inner.service_ctl.signal(()).await?;
        Ok(())
    }

    /// Triggers a disconnection on the underlying WebSocket.
    /// This is intended for debug purposes only.
    /// Can be used to test application reconnection logic.
    pub fn trigger_abort(&self) -> Result<()> {
        Ok(self.inner.rpc_client.trigger_abort()?)
    }

    pub async fn negotiate(&self, _network_id: &NetworkId) -> Result<GetStatusResponse> {
        let status = self.get_status().await?;
        // if status.network_id != *network_id {
        //     return Err(Error::NetworkId {
        //         expected: network_id.to_string(),
        //         // connected: status.network_id.to_string(),
        //     });
        // }
        Ok(status)
    }

    // pub fn notification_channel_receiver(&self) -> Receiver<Notification> {
    //     self.inner
    //         .notification_intake_channel
    //         .lock()
    //         .unwrap()
    //         .receiver
    //         .clone()
    // }
}

impl MonitorRpcClient {
    build_wrpc_client_interface!(RpcApiOps, [Ping, GetStatus]);

    pub async fn ping(&self) -> Result<PingResponse> {
        let request = PingRequest {};
        Ok(self.ping_call(request).await?)
    }

    pub async fn get_status(&self) -> Result<GetStatusResponse> {
        let request = GetStatusRequest {};
        Ok(self.get_status_call(request).await?)
    }
}
