#![allow(unused)]

use std::time::Duration;

const TIMEOUT: Duration = Duration::from_millis(5000);

/// Prepares for system reboot, including killing all processes, syncing disks and unmounting filesystems.
pub(crate) async fn prepare() {
    kill_all(TIMEOUT).await;
    sync_disks();
    _ = umount_all_filesystems().await;
}

/// Sends a signal to all running processes, then wait for them to be terminated. If the timeout expired, the processes are
/// force-killed.
pub(crate) async fn kill_all(timeout: Duration) {
    eprintln!("Sending SIGTERM to all processes");
    _ = kill_all_processes(libc::SIGTERM);

    eprintln!("Waiting for all processes to be terminated");
    let _lock = airupfx_process::lock().await;
    _ = tokio::time::timeout(
        timeout,
        tokio::task::spawn_blocking(|| {
            let mut status = 0;
            while unsafe { libc::wait(&mut status) > 0 } {}
        }),
    )
    .await;
    drop(_lock);

    eprintln!("Sending SIGKILL to all processes");
    _ = kill_all_processes(libc::SIGKILL);
}

/// Flushes the block buffer cache and synchronizes disk caches.
fn sync_disks() {
    unsafe {
        libc::sync();
    }
}

/// Sends the specified signal to all processes in the system.
fn kill_all_processes(signum: libc::c_int) -> std::io::Result<()> {
    let x = unsafe { libc::kill(-1, signum) };
    match x {
        0 => Ok(()),
        _ => Err(std::io::Error::last_os_error()),
    }
}

/// Unmounts all filesystems.
async fn umount_all_filesystems() -> std::io::Result<()> {
    airupfx_process::Command::new("umount")
        .arg("-a")
        .spawn()
        .await?
        .wait()
        .await
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::PermissionDenied))?;

    Ok(())
}
