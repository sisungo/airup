//! Filesystem utilities.

use std::path::Path;
use tokio::io::AsyncReadExt;

/// Like [`tokio::fs::read_to_string`], but max size of file is limited.
pub async fn read_to_string_limited(
    path: impl AsRef<Path>,
    limit: usize,
) -> std::io::Result<String> {
    let file = tokio::fs::File::open(path).await?;
    let mut capacity = file.metadata().await?.len() as usize;
    if capacity > limit {
        capacity = limit;
    }
    let mut buffer = String::with_capacity(capacity);
    file.take(limit as _).read_to_string(&mut buffer).await?;
    Ok(buffer)
}

/// Sets a file with socket permissions.
pub async fn set_sock_permission<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    crate::sys::fs::set_sock_permission(path).await
}
