//! Extension to the standard library.

use std::{future::Future, pin::Pin};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// An extension for standard [Result] type to support logging.
pub trait ResultExt<T> {
    /// Returns the contained `Ok` value, consuming the `self` value.
    fn unwrap_log(self, why: &str) -> T;
}
impl<T, E> ResultExt<T> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn unwrap_log(self, why: &str) -> T {
        match self {
            Ok(val) => val,
            Err(err) => {
                tracing::error!(target: "console", "{}: {}", why, err);
                crate::process::emergency();
            }
        }
    }
}

/// An extension for standard [Option] type to support logging.
pub trait OptionExt<T> {
    /// Returns the contained `Ok` value, consuming the `self` value.
    fn unwrap_log(self, why: &str) -> T;
}
impl<T> OptionExt<T> for Option<T> {
    fn unwrap_log(self, why: &str) -> T {
        self.map_or_else(
            || {
                tracing::error!(target: "console", "{why}");
                crate::process::emergency();
            },
            |val| val,
        )
    }
}
