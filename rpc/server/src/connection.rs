use crate::imports::*;
use kaspa_monitor_nexus::context::ContextT;
use kaspa_monitor_nexus::result::Result as NexusResult;
use std::fmt;

#[derive(Debug)]
struct ConnectionInner {
    pub id: u64,
    pub peer: SocketAddr,
    pub messenger: Arc<Messenger>,
}

impl ConnectionInner {
    // fn send(&self, message: Message) -> crate::result::Result<()> {
    //     Ok(self.messenger.send_raw_message(message)?)
    // }
}

// impl Notify<Notification> for ConnectionInner {
//     fn notify(&self, notification: Notification) -> NotifyResult<()> {
//         self.send(Connection::into_message(&notification, &self.messenger.encoding().into()))
//             .map_err(|err| NotifyError::General(err.to_string()))
//     }
// }

#[derive(Debug, Clone)]
pub struct Connection {
    inner: Arc<ConnectionInner>,
}

impl Connection {
    pub fn new(id: u64, peer: &SocketAddr, messenger: Arc<Messenger>) -> Connection {
        Connection {
            inner: Arc::new(ConnectionInner {
                id,
                peer: *peer,
                messenger,
            }),
        }
    }

    /// Obtain the connection id
    pub fn id(&self) -> u64 {
        self.inner.id
    }

    /// Get a reference to the connection [`Messenger`]
    pub fn messenger(&self) -> &Arc<Messenger> {
        &self.inner.messenger
    }

    pub fn peer(&self) -> &SocketAddr {
        &self.inner.peer
    }

    /// Creates a WebSocket [`Message`] that can be posted to the connection ([`Messenger`]) sink
    /// directly.
    pub fn create_serialized_notification_message<Ops, Msg>(
        encoding: Encoding,
        op: Ops,
        msg: Msg,
    ) -> WrpcResult<Message>
    where
        Ops: OpsT,
        Msg: MsgT,
    {
        match encoding {
            Encoding::Borsh => {
                workflow_rpc::server::protocol::borsh::create_serialized_notification_message(
                    op, msg,
                )
            }
            Encoding::SerdeJson => {
                workflow_rpc::server::protocol::borsh::create_serialized_notification_message(
                    op, msg,
                )
            }
        }
    }
}

impl fmt::Display for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.inner.id, self.inner.peer)
    }
}

#[async_trait::async_trait]
impl ContextT for Connection {
    fn id(&self) -> u64 {
        self.inner.id
    }

    async fn notify(&self, notification: Notification) -> NexusResult<()> {
        let message = Connection::create_serialized_notification_message(
            self.messenger().encoding(),
            RpcApiOps::Notify,
            Serializable(notification),
        )?;
        self.inner.messenger.send_raw_message(message)?;
        Ok(())
    }
}
