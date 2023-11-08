//! Inspection and manipulation of the process's environment.

use std::path::Path;

pub async fn setup_stdio<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    todo!();
}
