use crate::imports::*;

#[derive(Clone, Debug)]
pub enum Event {
    Start,
    Update,
    Status { status: Arc<Status> },
    Caps { uid: u64, caps: Arc<Caps> },
}
