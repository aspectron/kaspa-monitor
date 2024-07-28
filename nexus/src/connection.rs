use crate::imports::*;

#[allow(dead_code)]
pub const BIAS_SCALE: u64 = 1_000_000;

impl fmt::Display for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:016x}:{:016x}] [{:>4}] {}",
            self.sid(),
            self.uid(),
            self.sockets(),
            self.node.address
        )
    }
}

#[derive(Debug)]
pub struct Connection {
    args: Arc<Args>,
    caps: ArcSwapOption<Caps>,
    is_synced: AtomicBool,
    sid: AtomicU64,
    clients: AtomicU64,
    peers: AtomicU64,
    node: Arc<NodeConfig>,
    monitor: Arc<Monitor>,
    client: rpc::Client,
    shutdown_ctl: DuplexChannel<()>,
    delegate: ArcSwap<Option<Arc<Connection>>>,
    is_connected: AtomicBool,
    is_online: AtomicBool,
    sender: Sender<Event>,
}

impl Connection {
    pub fn try_new(
        args: &Arc<Args>,
        monitor: Arc<Monitor>,
        node: Arc<NodeConfig>,
        sender: Sender<Event>,
    ) -> Result<Self> {
        let client = match node.transport_kind {
            TransportKind::WrpcBorsh => {
                rpc::kaspa::Client::try_new(WrpcEncoding::Borsh, &node.address)?
            }
            TransportKind::WrpcJson => {
                rpc::kaspa::Client::try_new(WrpcEncoding::SerdeJson, &node.address)?
            }
            TransportKind::Grpc => {
                unimplemented!("gRPC support is not currently implemented")
            }
        };

        let client = rpc::Client::from(client);

        Ok(Self {
            args: args.clone(),
            caps: ArcSwapOption::new(None),
            monitor,
            node,
            client,
            shutdown_ctl: DuplexChannel::oneshot(),
            delegate: ArcSwap::new(Arc::new(None)),
            is_connected: AtomicBool::new(false),
            is_synced: AtomicBool::new(false),
            sid: AtomicU64::new(0),
            clients: AtomicU64::new(0),
            peers: AtomicU64::new(0),
            is_online: AtomicBool::new(false),
            sender,
        })
    }

    #[inline]
    pub fn verbose(&self) -> bool {
        self.args.verbose
    }

    #[inline]
    pub fn score(&self) -> u64 {
        self.clients.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn is_available(&self) -> bool {
        self.is_delegate()
            && self.online()
            && self.caps.load().as_ref().as_ref().is_some_and(|caps| {
                let clients = self.clients();
                let peers = self.peers();
                clients < caps.clients_limit && clients + peers < caps.fd_limit
            })
    }

    #[inline]
    pub fn connected(&self) -> bool {
        self.is_connected.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn online(&self) -> bool {
        self.is_online.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn is_synced(&self) -> bool {
        self.is_synced.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn clients(&self) -> u64 {
        self.clients.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn peers(&self) -> u64 {
        self.peers.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn sockets(&self) -> u64 {
        self.clients() + self.peers()
    }

    #[inline]
    pub fn caps(&self) -> Option<Arc<Caps>> {
        self.caps.load().clone()
    }

    #[inline]
    pub fn uid(&self) -> u64 {
        self.node.uid()
    }

    #[inline]
    pub fn sid(&self) -> u64 {
        self.sid.load(Ordering::Relaxed)
        // self.caps
        //     .load()
        //     .as_ref()
        //     .map(|caps| caps.system_id)
        //     .unwrap_or_default()
    }

    #[inline]
    pub fn address(&self) -> &str {
        self.node.address.as_str()
    }

    #[inline]
    pub fn node(&self) -> &Arc<NodeConfig> {
        &self.node
    }

    #[inline]
    pub fn network_id(&self) -> NetworkId {
        self.node.network
    }

    #[inline]
    pub fn is_delegate(&self) -> bool {
        self.delegate.load().is_none()
    }

    #[inline]
    pub fn delegate(self: &Arc<Self>) -> Arc<Connection> {
        match (**self.delegate.load()).clone() {
            Some(delegate) => delegate.delegate(),
            None => self.clone(),
        }
    }

    #[inline]
    pub fn bind_delegate(&self, delegate: Option<Arc<Connection>>) {
        self.delegate.store(Arc::new(delegate));
    }

    pub fn resolve_delegates(self: &Arc<Self>) -> Vec<Arc<Connection>> {
        let mut delegates = Vec::new();
        let mut delegate = (*self).clone();
        while let Some(next) = (**delegate.delegate.load()).clone() {
            delegates.push(next.clone());
            delegate = next;
        }
        delegates
    }

    pub fn status(&self) -> &'static str {
        if self.connected() {
            if !self.is_delegate() {
                "delegator"
            } else if self.is_synced() {
                "online"
            } else {
                "syncing"
            }
        } else {
            "offline"
        }
    }

    async fn connect(&self) -> Result<()> {
        self.client.connect().await?;
        Ok(())
    }

    async fn task(self: Arc<Self>) -> Result<()> {
        self.connect().await?;
        let rpc_ctl_channel = self.client.multiplexer().channel();
        let shutdown_ctl_receiver = self.shutdown_ctl.request.receiver.clone();
        let shutdown_ctl_sender = self.shutdown_ctl.response.sender.clone();

        // let mut ttl = sleep(TtlSettings::period());
        let mut ttl = TtlSettings::ttl();
        // TODO - delegate state changes inside `update_state()`!
        let mut poll = if self.is_delegate() {
            // workflow_core::task::
            interval(SyncSettings::poll())
        } else {
            // workflow_core::task::
            interval(SyncSettings::ping())
        };

        let mut last_connect_time: Option<Instant> = None;

        // use futures::StreamExt;
        loop {
            select! {

                _ = poll.next().fuse() => {

                    if TtlSettings::enable() {
                        if let Some(t) = last_connect_time {
                            if t.elapsed() > ttl {
                                // println!("-- t.elapsed(): {}", t.elapsed().as_millis());
                                last_connect_time = None;
                                // TODO reset caps ON ALL DELEGATES?
                                self.caps.store(None);
                                if self.is_connected.load(Ordering::Relaxed) {
                                    // log_info!("TTL","ttl disconnecting {}", self.node.address);
                                    self.client.disconnect().await.ok();
                                    // log_info!("TTL","Connecting {}", self.node.address);
                                    self.client.connect().await.ok();
                                }
                                continue;
                            }
                        }
                    }

                    if self.is_connected.load(Ordering::Relaxed) {
                        let previous = self.is_online.load(Ordering::Relaxed);
                        let online = self.update_state().await.is_ok();
                        self.is_online.store(online, Ordering::Relaxed);
                        if online != previous && self.verbose() {
                            if online {
                                log_info!("Online: {}", self.node.address);
                            } else {
                                log_error!("Offline: {}", self.node.address);
                            }
                        }
                    }
                }

                msg = rpc_ctl_channel.receiver.recv().fuse() => {
                    match msg {
                        Ok(msg) => {

                            // handle wRPC channel connection and disconnection events
                            match msg {
                                Ctl::Connect => {
                                    last_connect_time = Some(Instant::now());
                                    ttl = TtlSettings::ttl();
                                    if self.args.verbose {
                                        log_info!("Connected: {} - ttl: {:1.2}",self.node.address,ttl.as_secs() as f64 / 60.0 / 60.0);
                                    } else {
                                        log_info!("Connected: {}",self.node.address);
                                    }
                                    self.is_connected.store(true, Ordering::Relaxed);
                                    if self.update_state().await.is_ok() {
                                        self.is_online.store(true, Ordering::Relaxed);
                                    } else {
                                        self.is_online.store(false, Ordering::Relaxed);
                                    }
                                },
                                Ctl::Disconnect => {
                                    self.is_connected.store(false, Ordering::Relaxed);
                                    self.is_online.store(false, Ordering::Relaxed);
                                    last_connect_time = None;
                                    log_error!("Disconnected: {}",self.node.address);
                                }
                            }
                        }
                        Err(err) => {
                            println!("Monitor: error while receiving rpc_ctl_channel message: {err}");
                            break;
                        }
                    }
                }

                _ = shutdown_ctl_receiver.recv().fuse() => {
                    break;
                },

            }
        }

        shutdown_ctl_sender.send(()).await.unwrap();

        Ok(())
    }

    pub fn start(self: &Arc<Self>) -> Result<()> {
        let this = self.clone();
        tokio::spawn(async move {
            if let Err(error) = this.task().await {
                println!("NodeConnection task error: {:?}", error);
            }
        });

        Ok(())
    }

    pub async fn stop(self: &Arc<Self>) -> Result<()> {
        self.shutdown_ctl
            .signal(())
            .await
            .expect("NodeConnection shutdown signal error");
        Ok(())
    }

    async fn update_state(self: &Arc<Self>) -> Result<()> {
        if !self.is_delegate() {
            if let Err(err) = self.client.ping().await {
                log_error!("Ping: {err}");
            }
            return Ok(());
        }

        if self.caps().is_none() {
            let caps = self.client.get_caps().await?;
            let sid = caps.system_id();
            self.sid.store(sid, Ordering::Relaxed);
            let delegate_key = Delegate::new(sid, self.network_id());
            self.caps.store(Some(Arc::new(caps)));
            let mut delegates = self.monitor.delegates().write().unwrap();
            if let Some(delegate) = delegates.get(&delegate_key) {
                self.bind_delegate(Some(delegate.clone()));
            } else {
                delegates.insert(delegate_key, self.clone());
                self.bind_delegate(None);
            }
        }

        match self.client.get_sync().await {
            Ok(is_synced) => {
                let _previous_sync = self.is_synced.load(Ordering::Relaxed);
                self.is_synced.store(is_synced, Ordering::Relaxed);

                match self.client.get_status(self).await {
                    Ok(status) => {
                        self.sender
                            .send(Event::Status {
                                status: Arc::new(status),
                            })
                            .await
                            .unwrap();
                        Ok(())
                    }
                    Err(err) => {
                        log_error!("RPC: {self}");
                        log_error!("Error: {err}");
                        Err(Error::Metrics)
                    }
                }
            }
            Err(err) => {
                log_error!("RPC: {self}");
                log_error!("Error: {err}");
                Err(Error::Status)
            }
        }
    }
}
