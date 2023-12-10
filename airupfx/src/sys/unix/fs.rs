use std::{fs::Permissions, os::unix::prelude::PermissionsExt, path::Path};

/// Sets a file with socket permissions.
pub async fn set_sock_permission<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let perm = match *crate::process::ID {
        1 => Permissions::from_mode(0o770),
        _ => Permissions::from_mode(0o700),
    };
    tokio::fs::set_permissions(path.as_ref(), perm).await?;
    let path = path.as_ref().to_owned();
    if *crate::process::ID == 1 {
        let gid = None;
        tokio::task::spawn_blocking(move || std::os::unix::fs::chown(path, None, gid))
            .await
            .unwrap()?;
    }
    Ok(())
}

/// Commits filesystem caches to disk.
pub fn sync() {
    unsafe {
        libc::sync();
    }
}
