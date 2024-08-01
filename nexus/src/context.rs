use crate::imports::*;

#[async_trait]
pub trait ContextT: Send + Sync {
    fn id(&self) -> u64;
    async fn notify(&self, notification: Notification) -> Result<()>;
}
