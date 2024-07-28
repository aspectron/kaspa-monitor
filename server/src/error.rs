use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error: {0}")]
    Custom(String),

    #[error(transparent)]
    Core(#[from] kaspa_monitor_core::error::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Nexus(#[from] kaspa_monitor_nexus::error::Error),

    // #[error(transparent)]
    // Http(#[from] kaspa_monitor_http_server::error::Error),
    #[error(transparent)]
    RpcCore(#[from] kaspa_monitor_rpc_core::error::Error),

    #[error(transparent)]
    RpcServer(#[from] kaspa_monitor_rpc_server::error::Error),
}

impl Error {
    pub fn custom<T: Into<String>>(msg: T) -> Self {
        Error::Custom(msg.into())
    }
}
