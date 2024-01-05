use std::{fs::Permissions, os::unix::prelude::PermissionsExt, path::Path};

/// Sets a file with socket permissions.
pub async fn set_sock_permission<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let gid = None;
    if *crate::process::ID == 1 {
        let path = path.as_ref().to_owned();
        /* get `gid`: waiting for upstream update */
        tokio::task::spawn_blocking(move || std::os::unix::fs::chown(path, None, gid))
            .await
            .unwrap()?;
    }
    let perm = match (*crate::process::ID, gid) {
        (1, Some(_)) => Permissions::from_mode(0o770),
        _ => Permissions::from_mode(0o700),
    };
    tokio::fs::set_permissions(path.as_ref(), perm).await?;
    Ok(())
}

/// Commits filesystem caches to disk.
pub fn sync() {
    unsafe {
        libc::sync();
    }
}
