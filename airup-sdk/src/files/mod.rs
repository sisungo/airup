//! Definitions of Airup's file formats.

pub mod milestone;
pub mod policy;
pub mod service;
pub mod system_conf;

pub use milestone::Milestone;
pub use policy::Policy;
pub use service::Service;
pub use system_conf::SystemConf;

use crate::prelude::*;
use std::{borrow::Cow, sync::Arc};

pub trait Validate {
    fn validate(&self) -> Result<(), ReadError>;
}

pub trait Named {
    fn set_name(&mut self, name: String);
}

pub fn merge(doc: &mut toml::Value, patch: &toml::Value) {
    if !patch.is_table() {
        *doc = patch.clone();
        return;
    }

    if !doc.is_table() {
        *doc = toml::Value::Table(toml::Table::new());
    }
    let map = doc.as_table_mut().unwrap();
    for (key, value) in patch.as_table().unwrap() {
        if value.is_table() && value.as_table().unwrap().is_empty() {
            map.remove(key.as_str());
        } else {
            merge(
                map.entry(key.as_str())
                    .or_insert(toml::Value::Table(toml::Table::new())),
                value,
            );
        }
    }
}

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
                std::io::ErrorKind::NotFound => crate::Error::NotFound,
                _ => crate::Error::Io {
                    message: err.to_string(),
                },
            },
            Self::Parse(x) => crate::Error::BadObject { message: x.into() },
            Self::Validation(x) => crate::Error::BadObject { message: x },
        }
    }
}
