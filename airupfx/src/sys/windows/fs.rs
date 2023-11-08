use std::path::Path;

/// Sets a file with socket permissions.
pub async fn set_sock_permission<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    todo!()
}

/// Commits filesystem caches to disk.
pub async fn sync() {
    todo!()
}
