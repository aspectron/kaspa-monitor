use crate::imports::*;

#[derive(
    Describe,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
)]
#[borsh(use_discriminant = true)]
pub enum RpcApiOps {
    Notify = 0,
    Ping,
    GetStatus,
}
