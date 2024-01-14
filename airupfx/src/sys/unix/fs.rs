use crate::fs::Permission;
use std::{fs::Permissions, os::unix::prelude::PermissionsExt, path::Path};

pub async fn set_permission(path: &Path, perm: Permission) -> std::io::Result<()> {
    match perm {
        Permission::Socket => set_sock_permission(path).await,
        Permission::Lock => set_lock_permission(path).await,
    }
}

async fn set_sock_permission(path: &Path) -> std::io::Result<()> {
    let gid = None;
    if std::process::id() == 1 {
        let path = path.to_owned();
        /* get `gid`: waiting for upstream update */
        tokio::task::spawn_blocking(move || std::os::unix::fs::chown(path, None, gid))
            .await
            .unwrap()?;
    }
    let perm = match (std::process::id(), gid) {
        (1, Some(_)) => Permissions::from_mode(0o770),
        _ => Permissions::from_mode(0o700),
    };
    tokio::fs::set_permissions(path, perm).await?;
    Ok(())
}

async fn set_lock_permission(path: &Path) -> std::io::Result<()> {
    tokio::fs::set_permissions(path, Permissions::from_mode(0o444)).await
}

/// Commits filesystem caches to disk.
pub fn sync() {
    unsafe {
        libc::sync();
    }
}
