//! Filesystem utilities.

cfg_if::cfg_if! {
    if #[cfg(target_family = "unix")] {
        #[path = "unix.rs"]
        mod sys;
    } else {
        std::compile_error!("This target is not supported by `Airup` yet. Consider opening an issue at https://github.com/sisungo/airup/issues?");
    }
}

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
    sys::set_permission(path.as_ref(), perm).await
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
        _ = set_permission(&path, Permission::Lock).await;

        Ok(Self {
            holder: Some(holder.into_std().await),
            path,
        })
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        drop(self.holder.take());
        _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn lock() {
        let path = std::path::Path::new("/tmp/.airupfx-fs.test.lock");
        _ = tokio::fs::remove_file(path).await;
        let lock = crate::Lock::new(path.into()).await.unwrap();
        assert_eq!(
            tokio::fs::read_to_string(path).await.unwrap(),
            std::process::id().to_string()
        );
        drop(lock);
        assert!(!path.exists());
    }
}
