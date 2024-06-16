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

/// Registers a signal handler.
///
/// # Errors
/// An `Err(_)` is returned if the underlying OS function failed.
pub fn signal<
    F: FnOnce(i32) -> T + Clone + Send + Sync + 'static,
    T: Future<Output = ()> + Send + 'static,
>(
    signum: i32,
    op: F,
) -> std::io::Result<()> {
    let mut signal = tokio::signal::unix::signal(SignalKind::from_raw(signum))?;
    tokio::spawn(async move {
        loop {
            signal.recv().await;
            op.clone()(signum).await;
        }
    });

    Ok(())
}

/// Ignores a signal.
///
/// # Errors
/// An `Err(_)` is returned if the underlying OS function failed.
pub fn ignore(signum: i32) -> std::io::Result<()> {
    // Why not using `SIG_IGN`: it is by default inherited by child processes.
    signal(signum, |_| async {})
}

/// Initializes necessary primitives.
pub fn init() {
    if std::process::id() == 1 {
        for signum in [
            libc::SIGSEGV,
            libc::SIGBUS,
            libc::SIGILL,
            libc::SIGFPE,
            libc::SIGABRT,
        ] {
            unsafe {
                libc::signal(signum, fatal_error_handler as *const u8 as usize);
            }
        }
    }
}

/// A signal handler that handles fatal errors, like `SIGSEGV`, `SIGABRT`, etc.
///
/// This signal handler is only registered when we are `pid == 1`. It will firstly set `SIGCHLD` to `SIG_IGN`, so the kernel
/// would reap child processes, then print an error message to stderr. After that, this will hang the process forever.
extern "C" fn fatal_error_handler(signum: libc::c_int) {
    let begin = b"airupd[1]: caught ";
    let signal = match signum {
        libc::SIGSEGV => &b"SIGSEGV"[..],
        libc::SIGBUS => &b"SIGBUS"[..],
        libc::SIGILL => &b"SIGILL"[..],
        libc::SIGFPE => &b"SIGFPE"[..],
        libc::SIGABRT => &b"SIGABRT"[..],
        _ => &b"unknown signal"[..],
    };
    let end = b", this is a fatal error.\n";

    unsafe {
        libc::signal(SIGCHLD, libc::SIG_IGN);
        libc::write(2, begin.as_ptr() as *const _, begin.len());
        libc::write(2, signal.as_ptr() as *const _, signal.len());
        libc::write(2, end.as_ptr() as *const _, end.len());
    }

    loop {
        std::hint::spin_loop();
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn ignore() {
        super::ignore(super::SIGUSR1).unwrap();
    }
}
