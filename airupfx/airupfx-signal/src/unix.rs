use libc::c_int;
use std::future::Future;
use tokio::signal::unix::SignalKind;

/// Terminal line hangup.
pub const SIGHUP: c_int = libc::SIGHUP;

/// Interrupt program.
pub const SIGINT: c_int = libc::SIGINT;

/// Quit program.
pub const SIGQUIT: c_int = libc::SIGQUIT;

/// Write on a pipe with no reader.
pub const SIGPIPE: c_int = libc::SIGPIPE;

/// Software termination signal.
pub const SIGTERM: c_int = libc::SIGTERM;

/// Stop signal generated from keyboard.
pub const SIGTSTP: c_int = libc::SIGTSTP;

/// Child status has changed.
pub const SIGCHLD: c_int = libc::SIGCHLD;

/// Background read attempted from control terminal.
pub const SIGTTIN: c_int = libc::SIGTTIN;

/// Background write attempted to control terminal.
pub const SIGTTOU: c_int = libc::SIGTTOU;

/// I/O is possible on a descriptor (see fcntl(2)).
pub const SIGIO: c_int = libc::SIGIO;

/// User defined signal 1.
pub const SIGUSR1: c_int = libc::SIGUSR1;

/// User defined signal 2.
pub const SIGUSR2: c_int = libc::SIGUSR2;

/// Window size change
pub const SIGWINCH: c_int = libc::SIGWINCH;

/// Kills the process.
pub const SIGKILL: c_int = libc::SIGKILL;

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
            tokio::spawn(op.clone()(signum));
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
            libc::SIGSYS,
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
        libc::SIGSYS => &b"SIGSYS"[..],
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
