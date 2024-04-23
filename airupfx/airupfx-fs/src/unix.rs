use crate::Permission;
use std::{fs::Permissions, os::unix::prelude::PermissionsExt, path::Path};

pub async fn set_permission(path: &Path, perm: Permission) -> std::io::Result<()> {
    match perm {
        Permission::Socket => set_sock_permission(path).await,
        Permission::Lock => set_lock_permission(path).await,
    }
}

async fn set_sock_permission(path: &Path) -> std::io::Result<()> {
    tokio::fs::set_permissions(path, Permissions::from_mode(0o700)).await
}

async fn set_lock_permission(path: &Path) -> std::io::Result<()> {
    tokio::fs::set_permissions(path, Permissions::from_mode(0o444)).await
}
