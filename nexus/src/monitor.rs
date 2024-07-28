use crate::imports::*;

pub struct Monitor {
    args: Arc<Args>,
    connections: RwLock<AHashMap<u64, Arc<Connection>>>,
    delegates: RwLock<AHashMap<Delegate, Arc<Connection>>>,
    shutdown_ctl: DuplexChannel<()>,
    service: ServiceKind,
    sender: Sender<Event>,
}

impl fmt::Debug for Monitor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Monitor")
            .field("verbose", &self.verbose())
            // .field("connections", &self.connections)
            .finish()
    }
}

impl Monitor {
    pub fn new(args: &Arc<Args>, service: ServiceKind, sender: Sender<Event>) -> Self {
        Self {
            args: args.clone(),
            connections: Default::default(),
            delegates: Default::default(),
            shutdown_ctl: DuplexChannel::oneshot(),
            service,
            sender,
        }
    }

    pub fn verbose(&self) -> bool {
        self.args.verbose
    }

    pub fn delegates(&self) -> &RwLock<AHashMap<Delegate, Arc<Connection>>> {
        &self.delegates
    }

    pub fn connections(&self) -> AHashMap<u64, Arc<Connection>> {
        self.connections.read().unwrap().clone()
    }

    pub fn to_vec(&self) -> Vec<Arc<Connection>> {
        self.connections.read().unwrap().values().cloned().collect()
    }

    /// Process an update to `Server.toml` removing or adding node connections accordingly.
    pub async fn update_nodes(
        self: &Arc<Self>,
        global_node_list: &mut Vec<Arc<NodeConfig>>,
    ) -> Result<()> {
        let mut nodes = Vec::new();
        global_node_list.retain(|node| {
            if node.service() == self.service {
                nodes.push(node.clone());
                false
            } else {
                true
            }
        });

        let mut connections = self.connections();

        let create: Vec<_> = nodes
            .iter()
            // .filter(|node| !list.iter().any(|connection| connection.node() == **node))
            .filter(|node| !connections.contains_key(&node.uid()))
            .collect();

        let remove: Vec<_> = connections
            .iter()
            .filter_map(|(uid, connection)| {
                nodes
                    .iter()
                    .any(|node| node.uid() == *uid)
                    .then_some(connection)
            })
            .cloned()
            .collect();

        for node in create {
            let created = Arc::new(Connection::try_new(
                &self.args,
                self.clone(),
                node.clone(),
                self.sender.clone(),
            )?);
            created.start()?;
            connections.insert(created.node().uid(), created);
        }

        for removed in remove {
            removed.stop().await?;
            connections.remove(&removed.node().uid());
        }
        // }

        let targets = AHashMap::group_from(connections.values().map(|c| {
            (
                c.node().network_node_uid(),
                c.node().transport_kind(),
                c.clone(),
            )
        }));

        for (_network_uid, transport_map) in targets.iter() {
            if let Some(wrpc_borsh) = transport_map.get(&TransportKind::WrpcBorsh) {
                if let Some(wrpc_json) = transport_map.get(&TransportKind::WrpcJson) {
                    wrpc_json.bind_delegate(Some(wrpc_borsh.clone()));
                } else if let Some(grpc) = transport_map.get(&TransportKind::Grpc) {
                    grpc.bind_delegate(Some(wrpc_borsh.clone()));
                }
            }
        }

        *self.connections.write().unwrap() = connections;

        Ok(())
    }

    pub async fn start(self: &Arc<Self>) -> Result<()> {
        let this = self.clone();
        tokio::spawn(async move {
            if let Err(error) = this.task().await {
                println!("Monitor task error: {:?}", error);
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        self.shutdown_ctl
            .signal(())
            .await
            .expect("Monitor shutdown signal error");
        Ok(())
    }

    async fn task(self: Arc<Self>) -> Result<()> {
        let shutdown_ctl_receiver = self.shutdown_ctl.request.receiver.clone();
        let shutdown_ctl_sender = self.shutdown_ctl.response.sender.clone();

        #[allow(clippy::never_loop)]
        loop {
            select! {

                // _ = interval.next().fuse() => {
                //     for (params, sort) in self.sorts.iter() {
                //         if sort.load(Ordering::Relaxed) {
                //             sort.store(false, Ordering::Relaxed);

                //             let mut connections = self.connections.write().unwrap();
                //             if let Some(nodes) = connections.get_mut(params) {
                //                 nodes.sort_by_key(|connection| connection.score());
                //             }
                //         }
                //     }
                // }

                _ = shutdown_ctl_receiver.recv().fuse() => {
                    break;
                },

            }
        }

        shutdown_ctl_sender.send(()).await.unwrap();

        Ok(())
    }
}
