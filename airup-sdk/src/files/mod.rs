//! Definitions of Airup's file formats.

pub mod milestone;
pub mod service;
pub mod system_conf;
pub mod timer;

pub use milestone::Milestone;
pub use service::Service;
pub use system_conf::SystemConf;

use crate::prelude::*;
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
        Self::Parse(value.message().to_owned())
    }
}
impl From<serde_json::Error> for ReadError {
    fn from(value: serde_json::Error) -> Self {
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
impl IntoApiError for ReadError {
    fn into_api_error(self) -> crate::Error {
        match self {
            Self::Io(err) => match err.kind() {
                std::io::ErrorKind::NotFound => crate::Error::UnitNotFound,
                _ => crate::Error::Io {
                    message: err.to_string(),
                },
            },
            Self::Parse(x) => crate::Error::BadUnit { message: x.into() },
            Self::Validation(x) => crate::Error::BadUnit { message: x },
        }
    }
}
