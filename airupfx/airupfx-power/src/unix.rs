use std::time::Duration;

/// Sends a signal to all running processes, then wait for them to be terminated. If the timeout expired, the processes are
/// force-killed.
#[allow(unused)]
pub(crate) async fn kill_all(timeout: Duration) {
    eprintln!("Sending SIGTERM to all processes");
    unsafe {
        libc::kill(-1, libc::SIGTERM);
    }

    eprintln!("Waiting for all processes to be terminated");
    let _lock = airupfx_process::lock().await;
    tokio::time::timeout(
        timeout,
        tokio::task::spawn_blocking(|| {
            let mut status = 0;
            while unsafe { libc::wait(&mut status) > 0 } {}
        }),
    )
    .await
    .ok();
    drop(_lock);

    eprintln!("Sending SIGKILL to all processes");
    unsafe {
        libc::kill(-1, libc::SIGKILL);
    }
}
