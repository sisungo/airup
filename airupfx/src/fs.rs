//! Filesystem utilities.

use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

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

pub async fn set_permission<P: AsRef<Path>>(path: P, perm: Permission) -> std::io::Result<()> {
    crate::sys::fs::set_permission(path.as_ref(), perm).await
}

/// Represents to a lock file.
#[derive(Debug)]
pub struct Lock {
    holder: Option<std::fs::File>,
    path: PathBuf,
}
impl Lock {
    /// Creates an owned [`Lock`] instance for specified path.
    pub async fn new(path: PathBuf) -> std::io::Result<Self> {
        let mut options = tokio::fs::File::options();
        options.write(true);
        if std::process::id() != 1 {
            options.create_new(true);
        } else {
            options.create(true).truncate(true);
        }

        let mut holder = options.open(&path).await?;
        holder
            .write_all(std::process::id().to_string().as_bytes())
            .await?;
        set_permission(&path, Permission::Lock).await.ok();

        Ok(Self {
            holder: Some(holder.into_std().await),
            path,
        })
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        drop(self.holder.take());
        std::fs::remove_file(&self.path).ok();
    }
}
