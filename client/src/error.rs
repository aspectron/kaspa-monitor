use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Custom(String),

    #[error(transparent)]
    WorkflowEgui(#[from] workflow_egui::error::Error),

    #[error("{0}")]
    Eframe(String),
}

impl Error {
    pub fn custom<T: Into<String>>(msg: T) -> Self {
        Error::Custom(msg.into())
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::Custom(err.to_string())
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::Custom(err)
    }
}

impl From<eframe::Error> for Error {
    fn from(err: eframe::Error) -> Self {
        Error::Eframe(err.to_string())
    }
}
