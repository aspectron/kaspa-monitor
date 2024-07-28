use thiserror::Error;
use wasm_bindgen::JsValue;
use workflow_core::channel::{ChannelError, RecvError, SendError, TryRecvError, TrySendError};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error: {0}")]
    Custom(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ParseInt")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Channel send() error")]
    SendError,

    #[error("Channel recv() error")]
    RecvError,

    #[error("Channel try_send() error")]
    TrySendError,

    #[error("Channel try_recv() error")]
    TryRecvError,

    #[error("Channel error: {0}")]
    ChannelError(String),

    #[error("Invalid slice")]
    TryFromSlice(#[from] std::array::TryFromSliceError),

    #[error("Invalid JSON: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("API Version `{0}` is not supported")]
    ApiVersionNotSupported(u32),

    #[error("Invalid network id : {0}")]
    NetworkId(String),
}

impl Error {
    pub fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

impl From<Error> for JsValue {
    fn from(err: Error) -> Self {
        JsValue::from_str(&err.to_string())
    }
}

impl<T> From<SendError<T>> for Error {
    fn from(_: SendError<T>) -> Self {
        Error::SendError
    }
}

impl<T> From<TrySendError<T>> for Error {
    fn from(_: TrySendError<T>) -> Self {
        Error::TrySendError
    }
}

impl From<RecvError> for Error {
    fn from(_: RecvError) -> Self {
        Error::RecvError
    }
}

impl From<TryRecvError> for Error {
    fn from(_: TryRecvError) -> Self {
        Error::TryRecvError
    }
}

impl<T> From<ChannelError<T>> for Error {
    fn from(err: ChannelError<T>) -> Self {
        Error::ChannelError(err.to_string())
    }
}
