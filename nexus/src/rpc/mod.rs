pub mod kaspa;
pub mod sparkle;

use crate::imports::*;

const SOCKETS_PER_CORE: u64 = 1024;

#[enum_dispatch]
#[derive(Debug)]
pub enum Client {
    Kaspa(kaspa::Client),
    Sparkle(sparkle::Client),
}

#[allow(async_fn_in_trait)]
#[enum_dispatch(Client)]
pub trait ClientT: std::fmt::Debug + Sized + Send + Sync + 'static {
    fn multiplexer(&self) -> Multiplexer<Ctl> {
        unimplemented!()
    }

    async fn connect(&self) -> Result<()> {
        unimplemented!()
    }

    async fn disconnect(&self) -> Result<()> {
        unimplemented!()
    }

    async fn ping(&self) -> Result<()> {
        unimplemented!()
    }

    async fn get_caps(&self) -> Result<Caps> {
        unimplemented!()
    }

    async fn get_sync(&self) -> Result<bool> {
        unimplemented!()
    }

    async fn get_status(&self, _connection: &Arc<Connection>) -> Result<Status> {
        unimplemented!()
    }

    fn trigger_abort(&self) -> Result<()> {
        unimplemented!()
    }
}
