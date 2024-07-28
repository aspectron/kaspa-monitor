pub type Result<T> = std::result::Result<T, crate::error::Error>;
pub use workflow_rpc::client::result::Result as RpcResult;
