use super::Caps;
use crate::imports::*;
pub use kaspa_rpc_core::api::rpc::RpcApi;
use kaspa_rpc_core::{GetBlockDagInfoResponse, GetSystemInfoResponse};
pub use kaspa_wrpc_client::KaspaRpcClient;
// reduce fd_limit by this amount to ensure the
// system has enough file descriptors for other
// tasks (peers, db, etc)
// while default kHOST setup is:
// outgoing peers: 256
// incoming peers: 32
// peers are included the reported
// node connection count
// reserved for db etc.: 1024
const FD_MARGIN: u64 = 1024;

#[derive(Debug)]
pub struct Client {
    client: KaspaRpcClient,
    url: String,
    last_metrics_data: Mutex<Option<MetricsData>>,
}

impl Client {
    pub fn try_new(encoding: WrpcEncoding, url: &str) -> Result<Self> {
        let client = KaspaRpcClient::new(encoding, Some(url), None, None, None)?;

        Ok(Self {
            client,
            url: url.to_string(),
            last_metrics_data: Mutex::new(None),
        })
    }
}

impl rpc::ClientT for Client {
    fn multiplexer(&self) -> Multiplexer<Ctl> {
        self.client.ctl_multiplexer()
    }

    async fn connect(&self) -> Result<()> {
        let options = ConnectOptions {
            block_async_connect: false,
            strategy: ConnectStrategy::Retry,
            url: Some(self.url.clone()),
            ..Default::default()
        };

        self.client.connect(Some(options)).await?;
        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        Ok(self.client.disconnect().await?)
    }

    async fn ping(&self) -> Result<()> {
        Ok(self.client.ping().await?)
    }

    async fn get_caps(&self) -> Result<Caps> {
        let GetSystemInfoResponse {
            version,
            system_id,
            git_hash,
            cpu_physical_cores,
            total_memory,
            fd_limit,
        } = self.client.get_system_info().await?;
        let cpu_physical_cores = cpu_physical_cores as u64;
        let fd_limit = fd_limit as u64;
        // reduce node's fd_limit by FD_MARGIN to ensure
        // the system has enough file descriptors for other
        // tasks (peers, db, etc)
        let fd_limit_actual = fd_limit.checked_sub(FD_MARGIN).unwrap_or(32);
        // by default we assume that the node is able to accept
        // 1024 connections per core (default NGINX worker configuration)
        // TODO: this should be increased in the future once a custom
        // proxy is implemented
        let clients_limit = cpu_physical_cores * rpc::SOCKETS_PER_CORE;
        let system_id = system_id
            .and_then(|v| v[0..8].try_into().ok().map(u64::from_be_bytes))
            .unwrap_or_default();
        // let system_id_hex_string = format!("{:016x}", system_id);
        let git_hash = git_hash.as_ref().map(ToHex::to_hex);
        Ok(Caps {
            version,
            system_id,
            git_hash,
            total_memory,
            cpu_physical_cores,
            fd_limit: fd_limit_actual,
            clients_limit,
        })
    }

    async fn get_sync(&self) -> Result<bool> {
        Ok(self.client.get_sync_status().await?)
    }

    async fn get_status(&self, connection: &Arc<Connection>) -> Result<Status> {
        let metrics_response = self
            .client
            .get_metrics(true, true, true, true, true, false)
            .await?;
        let metrics_snapshot = {
            let current_metrics_data = MetricsData::try_from(metrics_response)?;
            let mut previous_metrics_data = self.last_metrics_data.lock().unwrap();
            let metrics_snapshot = previous_metrics_data
                .as_ref()
                .map(|previous_metrics_data| {
                    MetricsSnapshot::from((previous_metrics_data, &current_metrics_data))
                })
                .unwrap_or_default();
            previous_metrics_data.replace(current_metrics_data);
            metrics_snapshot
        };
        let GetBlockDagInfoResponse {
            network: network_id,
            block_count,
            header_count,
            tip_hashes,
            difficulty,
            past_median_time,
            virtual_parent_hashes,
            pruning_point_hash,
            virtual_daa_score,
            sink,
        } = self.client.get_block_dag_info().await?;

        let sid = connection.sid();
        let uid = connection.uid();
        let is_synced = connection.is_synced();

        let kaspa_node_status = KaspaNodeStatus {
            sid,
            uid,
            is_synced,
            // block_dag_info,
            metrics_snapshot,
            network_id,
            block_count,
            header_count,
            tip_hashes,
            difficulty,
            past_median_time,
            virtual_parent_hashes,
            pruning_point_hash,
            virtual_daa_score,
            sink,
        };

        Ok(kaspa_node_status.into())
    }

    fn trigger_abort(&self) -> Result<()> {
        Ok(self.client.trigger_abort()?)
    }
}
