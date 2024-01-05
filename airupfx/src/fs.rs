//! Filesystem utilities.

use std::path::Path;

/// Represents to a file permission.
#[derive(Debug, Clone, Copy)]
pub enum Permission {
    /// Permissions for socket files.
    /// 
    /// Socket files should only be accessed by current user, or if we are `pid == 1`, the `airup` group.
    Socket,

    /// Permissions for lock files.
    /// 
    /// Lock files should always be read, but never written.
    Lock,
}

/// Sets a file with socket permissions.
pub async fn set_sock_permission<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    crate::sys::fs::set_sock_permission(path).await
}
