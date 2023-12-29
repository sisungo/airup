//! Signal handling for Unix platforms.

use std::future::Future;
use tokio::signal::unix::SignalKind;

/// Terminal line hangup.
pub const SIGHUP: i32 = libc::SIGHUP;

/// Interrupt program.
pub const SIGINT: i32 = libc::SIGINT;

/// Quit program.
pub const SIGQUIT: i32 = libc::SIGQUIT;

/// Write on a pipe with no reader.
pub const SIGPIPE: i32 = libc::SIGPIPE;

/// Software termination signal.
pub const SIGTERM: i32 = libc::SIGTERM;

/// Stop signal generated from keyboard.
pub const SIGTSTP: i32 = libc::SIGTSTP;

/// Child status has changed.
pub const SIGCHLD: i32 = libc::SIGCHLD;

/// Background read attempted from control terminal.
pub const SIGTTIN: i32 = libc::SIGTTIN;

/// Background write attempted to control terminal.
pub const SIGTTOU: i32 = libc::SIGTTOU;

/// I/O is possible on a descriptor (see fcntl(2)).
pub const SIGIO: i32 = libc::SIGIO;

/// User defined signal 1.
pub const SIGUSR1: i32 = libc::SIGUSR1;

/// User defined signal 2.
pub const SIGUSR2: i32 = libc::SIGUSR2;

/// Window size change
pub const SIGWINCH: i32 = libc::SIGWINCH;

/// Kills the process.
pub const SIGKILL: i32 = libc::SIGKILL;

/// Segmentation fault.
pub const SIGSEGV: i32 = libc::SIGSEGV;

/// Aborted.
pub const SIGABRT: i32 = libc::SIGABRT;

/// Float-point exception.
pub const SIGFPE: i32 = libc::SIGFPE;

/// Registers a signal handler.
///
/// # Errors
/// An `Err(_)` is returned if the underlying OS function failed.
pub fn signal<
    F: FnMut(i32) -> T + Send + Sync + 'static,
    T: Future<Output = ()> + Send + 'static,
>(
    signum: i32,
    mut op: F,
) -> anyhow::Result<()> {
    let mut signal = tokio::signal::unix::signal(SignalKind::from_raw(signum))?;
    tokio::spawn(async move {
        loop {
            signal.recv().await;
            op(signum).await;
        }
    });

    Ok(())
}

/// Ignores a signal.
///
/// ## Errors
/// An `Err(_)` is returned if the underlying OS function failed.
pub fn ignore(signum: i32) -> std::io::Result<()> {
    let ret = unsafe { libc::signal(signum, libc::SIG_IGN) };
    if ret == libc::SIG_ERR {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
