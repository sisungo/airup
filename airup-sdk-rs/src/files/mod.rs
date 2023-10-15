pub mod milestone;
pub mod service;

pub use milestone::Milestone;
pub use service::Service;

use std::{borrow::Cow, sync::Arc};

#[derive(Debug, Clone, thiserror::Error)]
pub enum ReadError {
    #[error("{0}")]
    Io(Arc<std::io::Error>),

    #[error("{0}")]
    Parse(String),

    #[error("{0}")]
    Validation(Cow<'static, str>),
}
impl From<std::io::Error> for ReadError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.into())
    }
}
impl From<toml::de::Error> for ReadError {
    fn from(value: toml::de::Error) -> Self {
        Self::Parse(value.to_string())
    }
}
impl From<&'static str> for ReadError {
    fn from(value: &'static str) -> Self {
        Self::Validation(value.into())
    }
}
impl From<String> for ReadError {
    fn from(value: String) -> Self {
        Self::Validation(value.into())
    }
}
impl From<std::io::ErrorKind> for ReadError {
    fn from(value: std::io::ErrorKind) -> Self {
        Self::from(std::io::Error::from(value))
    }
}
