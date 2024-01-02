//! Filesystem utilities.

use std::path::Path;

/// Sets a file with socket permissions.
pub async fn set_sock_permission<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    crate::sys::fs::set_sock_permission(path).await
}
