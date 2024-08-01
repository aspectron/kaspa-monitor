use crate::imports::*;

struct Inner {
    args: Arc<Args>,
    kaspa: Arc<Monitor>,
    sparkle: Arc<Monitor>,
    shutdown_ctl: DuplexChannel<()>,
    events: Channel<Event>,
    contexts: RwLock<AHashMap<u64, Arc<dyn ContextT>>>,
}

impl Inner {
    fn new(args: &Arc<Args>) -> Self {
        let events = Channel::unbounded();
        Self {
            args: args.clone(),
            kaspa: Arc::new(Monitor::new(
                args,
                ServiceKind::Kaspa,
                events.sender.clone(),
            )),
            sparkle: Arc::new(Monitor::new(
                args,
                ServiceKind::Sparkle,
                events.sender.clone(),
            )),
            shutdown_ctl: DuplexChannel::oneshot(),
            events,
            contexts: RwLock::new(AHashMap::new()),
        }
    }
}

#[derive(Clone)]
pub struct Nexus {
    inner: Arc<Inner>,
}

impl Nexus {
    pub fn try_new(args: &Arc<Args>) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(Inner::new(args)),
        })
    }

    pub fn args(&self) -> &Arc<Args> {
        &self.inner.args
    }

    pub async fn start(self: &Arc<Self>) -> Result<()> {
        self.inner.kaspa.start().await?;
        self.inner.sparkle.start().await?;

        let this = self.clone();
        tokio::spawn(async move {
            if let Err(error) = this.task().await {
                println!("Resolver task error: {:?}", error);
            }
        });

        self.inner.events.send(Event::Start).await?;

        Ok(())
    }

    pub async fn stop(self: &Arc<Self>) -> Result<()> {
        self.inner.sparkle.stop().await?;
        self.inner.kaspa.stop().await?;

        self.inner
            .shutdown_ctl
            .signal(())
            .await
            .expect("Monitor shutdown signal error");

        Ok(())
    }

    async fn task(self: Arc<Self>) -> Result<()> {
        let events = self.inner.events.receiver.clone();
        let shutdown_ctl_receiver = self.inner.shutdown_ctl.request.receiver.clone();
        let shutdown_ctl_sender = self.inner.shutdown_ctl.response.sender.clone();

        let mut update = workflow_core::task::interval(Updates::duration());

        loop {
            select! {

                msg = events.recv().fuse() => {
                    match msg {
                        Ok(event) => {
                            match event {
                                Event::Start => {
                                    if let Err(err) = self.update(true).await {
                                        log_error!("Config [startup]: {err}");
                                    }
                                },
                                Event::Update => {
                                    if let Err(err) = self.update(false).await {
                                        log_error!("Config [update]: {err}");
                                    }
                                },
                                Event::Status { status } => {
                                    // println!("Status: {status:?}");
                                    let update = Update::Status { status };
                                    for context in self.contexts() {
                                        // println!("Posting `Status` to context: {}", context.id());
                                        context.notify(Notification::Update { update : update.clone() }).await.ok();
                                    }
                                },
                                Event::Caps { uid, caps } => {
                                    // println!("Caps: {uid} {caps:?}");
                                    let update = Update::Caps { uid, caps };
                                    for context in self.contexts() {
                                        // println!("Posting `Caps` to context: {}", context.id());
                                        context.notify(Notification::Update { update : update.clone() }).await.ok();
                                        // context.notify(update.clone()).await.ok();
                                    }

                                },
                            }
                        }
                        Err(err) => {
                            println!("Monitor: error while receiving events message: {err}");
                            break;
                        }

                    }
                }

                _ = update.next().fuse() => {
                    self.inner.events.send(Event::Update).await?;
                }

                _ = shutdown_ctl_receiver.recv().fuse() => {
                    break;
                },

            }
        }

        shutdown_ctl_sender.send(()).await.unwrap();

        Ok(())
    }

    async fn update_nodes(
        self: &Arc<Self>,
        mut global_node_list: Vec<Arc<NodeConfig>>,
    ) -> Result<()> {
        self.inner.kaspa.update_nodes(&mut global_node_list).await?;
        self.inner
            .sparkle
            .update_nodes(&mut global_node_list)
            .await?;

        for node in global_node_list.iter() {
            log_error!("Update: Dangling node record: {}", node);
        }
        Ok(())
    }

    async fn update(self: &Arc<Self>, fallback_to_local: bool) -> Result<()> {
        match update_global_config().await {
            Ok(Some(global_node_list)) => {
                self.update_nodes(global_node_list).await?;
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(_) if fallback_to_local => {
                let node_list = load_config()?;
                self.update_nodes(node_list).await?;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    // // respond with a JSON object containing the status of all nodes
    pub fn connections(&self) -> Vec<Arc<Connection>> {
        let kaspa = self.inner.kaspa.to_vec();

        let sparkle = self.inner.sparkle.to_vec();

        kaspa.into_iter().chain(sparkle).collect::<Vec<_>>()
    }

    pub async fn register_context(&self, context: Arc<dyn ContextT>) {
        println!("Registering context: {}", context.id());

        let connections = self.connections();
        let caps = connections
            .iter()
            .filter_map(|c| {
                let caps = c.caps()?;
                let uid = c.uid();
                Some((uid, caps))
            })
            .collect::<Vec<_>>();

        for (uid, caps) in caps {
            let update = Update::Caps { uid, caps };
            context
                .notify(Notification::Update {
                    update: update.clone(),
                })
                .await
                .ok();
        }

        let mut contexts = self.inner.contexts.write().unwrap();
        contexts.insert(context.id(), context.clone());
    }

    pub async fn unregister_context(&self, id: u64) {
        println!("Unregistering context: {}", id);

        let mut contexts = self.inner.contexts.write().unwrap();
        contexts.remove(&id);
    }

    pub fn contexts(&self) -> Vec<Arc<dyn ContextT>> {
        self.inner
            .contexts
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }
}

impl Nexus {
    pub async fn ping_call(
        &self,
        _ctx: &dyn ContextT,
        _request: PingRequest,
    ) -> Result<PingResponse> {
        println!();
        println!("+------+");
        println!("| PING |");
        println!("+------+");
        println!();

        let response = PingResponse {};
        Ok(response)
    }

    pub async fn get_status_call(
        &self,
        _ctx: &dyn ContextT,
        _request: GetStatusRequest,
    ) -> Result<GetStatusResponse> {
        let response = GetStatusResponse {
            kaspa_monitor_version: std::env!("CARGO_PKG_VERSION").to_string(),
            // network_id: self.network_id(),
        };
        Ok(response)
    }
}

#[async_trait]
impl Service for Nexus {
    async fn spawn(self: Arc<Self>, _runtime: Runtime) -> ServiceResult<()> {
        self.start().await.map_err(ServiceError::custom)?;
        Ok(())
    }

    fn terminate(self: Arc<Self>) {
        self.inner.shutdown_ctl.request.try_send(()).unwrap();
    }

    async fn join(self: Arc<Self>) -> ServiceResult<()> {
        self.stop().await.map_err(ServiceError::custom)?;
        Ok(())
    }
}
