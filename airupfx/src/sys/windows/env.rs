//! Inspection and manipulation of the process's environment.

use std::path::Path;
use sysinfo::{ProcessExt, SystemExt, Uid};

pub async fn setup_stdio<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    Ok(())
}

pub fn current_uid() -> sysinfo::Uid {
    let mut system = sysinfo::System::new();
    let pid = sysinfo::Pid::from(*crate::process::ID as usize);
    system.refresh_process(pid);
    system.process(pid).unwrap().user_id().unwrap().clone()
}
