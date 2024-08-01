use crate::imports::*;
use kaspa_monitor_rpc_client::prelude::{ConnectOptions, ConnectStrategy, MonitorRpcClient};

pub enum MonitorServiceEvents {
    Exit,
}

pub struct MonitorService {
    pub service_events: Channel<MonitorServiceEvents>,
    pub task_ctl: Channel<()>,
    pub rpc_client: MonitorRpcClient,
}

impl Default for MonitorService {
    fn default() -> Self {
        Self {
            service_events: Channel::unbounded(),
            task_ctl: Channel::oneshot(),
            rpc_client: MonitorRpcClient::default(),
        }
    }
}

#[async_trait]
impl Service for MonitorService {
    fn name(&self) -> &'static str {
        "monitor-service"
    }

    async fn spawn(self: Arc<Self>, runtime: Runtime) -> ServiceResult {
        let url = String::from("ws://localhost:6969");
        println!("Monitor Service connecting to {url}");

        let options = ConnectOptions {
            block_async_connect: false,
            strategy: ConnectStrategy::Retry,
            url: Some(url),
            ..Default::default()
        };
        self.rpc_client.connect(Some(options)).await.unwrap();

        let receiver = self.rpc_client.notification_channel_receiver();

        loop {
            select! {
                msg = receiver.recv().fuse() => {
                    if let Ok(event) = msg {

                        match event {
                            Notification::Update { update } => {
                                runtime.send(Event::Update { update }).await.unwrap();
                            }
                        }
                    } else {
                        break;
                    }
                }
                msg = self.as_ref().service_events.receiver.recv().fuse() => {
                    if let Ok(event) = msg {
                        match event {
                            MonitorServiceEvents::Exit => {
                                break;
                            }
                        }
                    } else {

                        break;
                    }
                }
            }
        }

        self.task_ctl.send(()).await.unwrap();
        Ok(())
    }

    fn terminate(self: Arc<Self>) {
        self.service_events
            .sender
            .try_send(MonitorServiceEvents::Exit)
            .unwrap();
    }

    async fn join(self: Arc<Self>) -> ServiceResult {
        self.task_ctl.recv().await.unwrap();
        Ok(())
    }
}
