use crate::imports::*;
use crate::runtime::Runtime;
use std::sync::Arc;

pub use crate::error::Error as ServiceError;
pub use crate::result::Result as ServiceResult;

#[async_trait]
pub trait Service: Sync + Send {
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Start the service
    async fn spawn(self: Arc<Self>, runtime: Runtime) -> Result<()>;

    /// Signal the service termination (post a shutdown request)
    fn terminate(self: Arc<Self>);

    /// Block until the service is terminated
    async fn join(self: Arc<Self>) -> Result<()>;
}
