use crate::{connection::*, server::*};
use kaspa_monitor_macros::build_wrpc_server_interface;
use kaspa_monitor_rpc_core::prelude::*;
use std::sync::Arc;
use workflow_rpc::server::prelude::*;

pub struct Router {
    pub interface: Arc<Interface<Server, Connection, RpcApiOps>>,
    pub server_context: Server,
}

impl Router {
    pub fn new(server_context: Server) -> Self {
        #[allow(unreachable_patterns)]
        let interface = build_wrpc_server_interface!(
            server_context.clone(),
            Server,
            Connection,
            RpcApiOps,
            [Ping, GetStatus]
        );

        Router {
            interface: Arc::new(interface),
            server_context,
        }
    }
}
