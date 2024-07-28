use crate::imports::*;
use crate::router::Router;
use crate::{connection::*, server::*};
use kaspa_monitor_core::runtime::*;
use kaspa_monitor_nexus::prelude::Nexus;
use kaspa_monitor_rpc_core::prelude::*;
pub use workflow_rpc::server::{Encoding as WrpcEncoding, WebSocketConfig, WebSocketCounters};

static MAX_WRPC_MESSAGE_SIZE: usize = 1024 * 1024 * 128; // 128MB

/// Options for configuring the wRPC server
pub struct WrpcOptions {
    pub listen_address: String,
    pub verbose: bool,
    pub encoding: WrpcEncoding,
}

impl Default for WrpcOptions {
    fn default() -> Self {
        WrpcOptions {
            listen_address: "127.0.0.1:7878".to_owned(),
            verbose: false,
            encoding: WrpcEncoding::Borsh,
        }
    }
}

impl WrpcOptions {
    pub fn listen(mut self, address: &str) -> Self {
        address.clone_into(&mut self.listen_address);
        self
    }
}

pub struct WrpcService {
    options: Arc<WrpcOptions>,
    rpc_server: RpcServer,
    shutdown: Channel<()>,
}

impl WrpcService {
    /// Create and initialize RpcServer
    pub async fn try_new(nexus: &Nexus, options: WrpcOptions) -> Result<Self> {
        let options = Arc::new(options);
        let server = Server::new(nexus, options.clone());

        // Create router (initializes Interface registering RPC method and notification handlers)
        let router = Arc::new(Router::new(server.clone()));
        // Create a server
        let rpc_server = RpcServer::new_with_encoding::<Server, Connection, RpcApiOps, Id64>(
            options.encoding,
            Arc::new(server.clone()),
            router.interface.clone(),
            None,
            true,
        );

        Ok(WrpcService {
            options,
            rpc_server,
            shutdown: Channel::oneshot(),
        })
    }
}

#[async_trait]
impl Service for WrpcService {
    async fn spawn(self: Arc<Self>, _runtime: Runtime) -> ServiceResult<()> {
        let listen_address = self.options.listen_address.clone();
        log_info!("wRPC server listening on: {}", listen_address);
        let listener = self
            .rpc_server
            .bind(listen_address.as_str())
            .await
            .map_err(ServiceError::custom)?;

        spawn(async move {
            let config = WebSocketConfig {
                max_message_size: Some(MAX_WRPC_MESSAGE_SIZE),
                ..Default::default()
            };
            let serve_result = self.rpc_server.listen(listener, Some(config)).await;
            match serve_result {
                Ok(_) => log_info!("wRPC Server stopped on: {}", listen_address),
                Err(err) => panic!("wRPC Server {listen_address} stopped with error: {err:?}"),
            }
        });

        Ok(())
    }

    fn terminate(self: Arc<Self>) {
        spawn(async move {
            self.rpc_server
                .stop()
                .unwrap_or_else(|err| log_warn!("wRPC unable to signal shutdown: `{err}`"));
            self.rpc_server
                .join()
                .await
                .unwrap_or_else(|err| log_warn!("wRPC error: `{err}"));

            self.shutdown.send(()).await.unwrap();
        });
    }

    async fn join(self: Arc<Self>) -> ServiceResult<()> {
        self.shutdown.recv().await?;
        Ok(())
    }
}
