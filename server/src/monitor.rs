use kaspa_monitor_core::runtime::Runtime;
use kaspa_monitor_nexus::prelude::Nexus;
use kaspa_monitor_rpc_server::{WrpcOptions, WrpcService};
use std::sync::Arc;
#[allow(unused_imports)]
use workflow_core::dirs::home_dir;

use crate::args::Args;
use crate::result::Result;

#[derive(Default)]
pub struct Server {}

impl Server {
    pub async fn run(&self, runtime: &Runtime) -> Result<()> {
        let args = Args::parse();

        if args.trace {
            workflow_log::set_log_level(workflow_log::LevelFilter::Trace);
        }

        kaspa_monitor_core::debug::enable(args.debug);

        // --- Services ---

        let nexus = Nexus::try_new(&Arc::new(args.as_ref().into()))
            .expect("Unable to create nexus instance.");
        runtime.bind(Arc::new(nexus.clone()));

        let wrpc_options = WrpcOptions::default().listen(args.rpc_listen.to_string().as_str());
        let wrpc_server = WrpcService::try_new(&nexus, wrpc_options)
            .await
            .expect("Unable to create wRPC service.");
        runtime.bind(Arc::new(wrpc_server));

        runtime.run().await?;

        Ok(())
    }
}
