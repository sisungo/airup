//! Extension to the standard library.

use std::{future::Future, pin::Pin};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// An extension for standard [`Result`] type to support logging.
pub trait ResultExt<T> {
    /// Returns the contained `Ok` value, consuming the `self` value.
    fn unwrap_log(self, why: &str) -> impl Future<Output = T>;
}
impl<T, E> ResultExt<T> for Result<T, E>
where
    E: std::fmt::Display,
{
    async fn unwrap_log(self, why: &str) -> T {
        match self {
            Ok(val) => val,
            Err(err) => {
                tracing::error!(target: "console", "{}: {}", why, err);
                if std::process::id() == 1 {
                    let ace = crate::ace::Ace::default();
                    loop {
                        if let Err(err) = ace.run_wait("/bin/sh").await {
                            tracing::error!(target: "console", "Failed to start `/bin/sh`: {err}");
                        }
                        if let Err(err) = crate::process::reload_image() {
                            tracing::error!(target: "console", "Failed to reload `airupd` process image: {err}");
                        }
                    }
                } else {
                    std::process::exit(1);
                }
            }
        }
    }
}
