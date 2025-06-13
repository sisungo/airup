//! Process management on Unix platforms.
//!
//! This internally registers a `SIGCHLD` listener and spawns a background task to listen the signal. When registering
//! a new child process (e.g. by spawning), the PID is subscribed from the internal table. When a new sucessful call to
//! `waitpid()` completed, if the PID was previously subscribed, the result will be sent to the subscriber and then the
//! subscription is cancelled.

#![allow(
    unstable_name_collisions,
    reason = "The names are to build Airup on stable Rust, since the methods are not stablized yet"
)]

use super::{CommandEnv, ExitStatus, Stdio, Wait};
use airupfx_io::line_piper::{self, CallbackGuard};
use std::{
    cmp,
    collections::HashMap,
    convert::Infallible,
    ffi::CStr,
    os::unix::{ffi::OsStrExt, process::CommandExt as _},
    path::Path,
    sync::{OnceLock, RwLock},
};
use tokio::{signal::unix::SignalKind, sync::watch};

/// Port of unstable feature `#![feature(process_setsid)]` and `#![feature(setgroups)]` to stable Rust.
///
/// This will be deleted when the features gets stablized, or moved if they are determined to remove.
pub trait CommandExt {
    /// View [Tracking Issue for `process_setsid`](https://github.com/rust-lang/rust/issues/105376).
    fn setsid(&mut self) -> &mut Self;

    /// View [`std::os::unix::process::CommandExt::groups`].
    fn groups(&mut self, groups: &[libc::gid_t]) -> &mut Self;

    /// View [`std::os::unix::process::CommandExt::uid`].
    ///
    /// This is stable in Rust, however, it prevents some functions requiring root, like `groups` from working, so here
    /// a version using `.pre_exec()` is provided.
    fn uid(&mut self, uid: libc::uid_t) -> &mut Self;

    /// Chroots to the specified directory.
    fn root_dir(&mut self, path: &Path) -> &mut Self;

    /// View [`std::process::Command::current_dir`].
    ///
    /// This is stable in Rust, however, it works weirdly when combined with `root_dir`, since it is applied earilier than
    /// any `.pre_exec()`.
    fn current_dir(&mut self, path: &Path) -> &mut Self;
}
impl CommandExt for std::process::Command {
    fn setsid(&mut self) -> &mut Self {
        fn setsid() -> std::io::Result<libc::pid_t> {
            unsafe {
                let pgid = libc::setsid();
                match pgid {
                    -1 => Err(std::io::Error::last_os_error()),
                    x => Ok(x),
                }
            }
        }

        unsafe { self.pre_exec(|| setsid().map(|_| ())) }
    }

    fn groups(&mut self, groups: &[libc::gid_t]) -> &mut Self {
        fn setgroups(groups: &[libc::gid_t]) -> std::io::Result<()> {
            unsafe {
                let pgid = libc::setgroups(groups.len() as _, groups.as_ptr()) as _;
                match pgid {
                    0 => Ok(()),
                    -1 => Err(std::io::Error::last_os_error()),
                    _ => unreachable!(),
                }
            }
        }

        let groups = groups.to_vec();
        unsafe { self.pre_exec(move || setgroups(&groups[..])) }
    }

    fn uid(&mut self, uid: libc::uid_t) -> &mut Self {
        fn setuid(uid: libc::uid_t) -> std::io::Result<()> {
            unsafe {
                match libc::setuid(uid) {
                    0 => Ok(()),
                    -1 => Err(std::io::Error::last_os_error()),
                    _ => unreachable!(),
                }
            }
        }

        unsafe { self.pre_exec(move || setuid(uid)) }
    }

    fn root_dir(&mut self, path: &Path) -> &mut Self {
        fn chroot(path: &Path) -> std::io::Result<()> {
            unsafe {
                let s = CStr::from_bytes_with_nul(path.as_os_str().as_bytes())
                    .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?
                    .as_ptr();
                match libc::chroot(s) {
                    0 => Ok(()),
                    -1 => Err(std::io::Error::last_os_error()),
                    _ => unreachable!(),
                }
            }
        }

        let path = path.to_path_buf();
        unsafe { self.pre_exec(move || chroot(&path)) }
    }

    fn current_dir(&mut self, path: &Path) -> &mut Self {
        let path = path.to_path_buf();
        unsafe { self.pre_exec(move || std::env::set_current_dir(&path)) }
    }
}

pub type Pid = libc::pid_t;

static CHILD_QUEUE: OnceLock<&'static ChildQueue> = OnceLock::new();

pub fn is_forking_supervisable() -> bool {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            true
        } else {
            std::process::id() == 1
        }
    }
}

pub fn reload_image() -> std::io::Result<Infallible> {
    Err(std::process::Command::new(std::env::current_exe()?)
        .args(std::env::args_os().skip(1))
        .exec())
}

/// Sends the given signal to the specified process.
///
/// # Errors
/// An `Err(_)` is returned if the underlying OS function failed.
fn kill(pid: Pid, signum: i32) -> std::io::Result<()> {
    let result = unsafe { libc::kill(pid as _, signum) };
    match result {
        0 => Ok(()),
        _ => Err(std::io::Error::last_os_error()),
    }
}

pub trait ExitStatusExt {
    /// Converts from a `status` returned by [`libc::waitpid`] to [`ExitStatus`].
    fn from_unix(status: libc::c_int) -> Self;
}
impl ExitStatusExt for ExitStatus {
    fn from_unix(status: libc::c_int) -> Self {
        if libc::WIFEXITED(status) {
            Self::Exited(libc::WEXITSTATUS(status))
        } else if libc::WIFSIGNALED(status) {
            Self::Signaled(libc::WTERMSIG(status))
        } else {
            Self::Other
        }
    }
}

macro_rules! map_stdio {
    ($fx:expr, $std:expr) => {
        match &$fx {
            Stdio::Callback(c) => Some(line_piper::set_callback($std, c.clone_boxed())),
            _ => None,
        }
    };
}

/// Representation of a running or exited child process.
#[derive(Debug)]
pub(crate) struct Child {
    pid: Pid,
    wait_queue: watch::Receiver<Option<Wait>>,
    _stdout_guard: Option<CallbackGuard>,
    _stderr_guard: Option<CallbackGuard>,
}
impl Child {
    pub(crate) const fn id(&self) -> Pid {
        self.pid
    }

    fn from_std(env: &CommandEnv, c: std::process::Child) -> Self {
        let pid = c.id();
        let _stdout_guard = c
            .stdout
            .and_then(|x| tokio::process::ChildStdout::from_std(x).ok())
            .and_then(|x| map_stdio!(env.stdout, x));
        let _stderr_guard = c
            .stderr
            .and_then(|x| tokio::process::ChildStderr::from_std(x).ok())
            .and_then(|x| map_stdio!(env.stderr, x));
        Self {
            pid: pid as _,
            wait_queue: child_queue().subscribe(pid as _),
            _stdout_guard,
            _stderr_guard,
        }
    }

    /// Converts from process ID to [`Child`].
    ///
    /// # Safety
    /// The behavior converting from an invalid child PID to [`Child`] is not guaranteed.
    unsafe fn from_pid_unchecked(pid: Pid) -> Self {
        Self {
            pid,
            wait_queue: child_queue().subscribe(pid),
            _stdout_guard: None,
            _stderr_guard: None,
        }
    }

    pub(crate) fn from_pid(pid: Pid) -> std::io::Result<Self> {
        (wait_nonblocking(pid)?).map_or_else(
            || Ok(unsafe { Self::from_pid_unchecked(pid) }),
            |_| Err(std::io::ErrorKind::NotFound.into()),
        )
    }

    pub(crate) async fn wait(&self) -> Result<Wait, WaitError> {
        let mut wait_queue = self.wait_queue.clone();
        let wait = wait_queue.wait_for(|x| x.is_some()).await.unwrap();
        Ok(wait.clone().unwrap())
    }

    pub(crate) async fn send_signal(&self, sig: i32) -> std::io::Result<()> {
        let _lock = lock().await;
        if self.wait_queue.borrow().is_none() {
            kill(self.pid, sig)
        } else {
            Err(std::io::ErrorKind::NotFound.into())
        }
    }

    pub(crate) async fn kill(&self) -> std::io::Result<()> {
        self.send_signal(libc::SIGKILL).await
    }
}
impl Drop for Child {
    fn drop(&mut self) {
        child_queue().unsubscribe(self.pid);
    }
}

/// A queue of waiting child processes.
#[derive(Debug, Default)]
struct ChildQueue {
    queue: RwLock<HashMap<Pid, watch::Sender<Option<Wait>>>>,
    lock: tokio::sync::Mutex<()>,
}
impl ChildQueue {
    /// Creates a new [`ChildQueue`] instance.
    #[must_use]
    fn new() -> Self {
        Self::default()
    }

    /// Starts the child queue task.
    fn start(&'static self) -> std::io::Result<tokio::task::JoinHandle<()>> {
        #[cfg(target_os = "linux")]
        {
            unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER) };
        }

        let mut signal = tokio::signal::unix::signal(SignalKind::child())?;
        Ok(tokio::spawn(async move {
            loop {
                signal.recv().await;
                loop {
                    let _lock = self.lock.lock().await;
                    let wait = match wait_nonblocking(-1) {
                        Ok(Some(x)) => x,
                        Ok(None) => break,
                        Err(_) => {
                            break;
                        }
                    };

                    if wait.code().is_some() || wait.signal().is_some() {
                        self.send(wait).await;
                        continue;
                    }
                }
            }
        }))
    }

    /// Creates a new [`watch::Receiver`] handle that will receive [`Wait`] sent after this call to `subscribe`.
    fn subscribe(&self, pid: Pid) -> watch::Receiver<Option<Wait>> {
        let mut lock = self.queue.write().unwrap();
        let (tx, rx) = watch::channel(None);
        lock.insert(pid, tx);
        rx
    }

    /// Removes a subscription ahead of time.
    fn unsubscribe(&self, pid: Pid) -> Option<()> {
        self.queue.write().unwrap().remove(&pid).map(|_| ())
    }

    /// Sends the given [`Wait`] to the queue.
    async fn send(&self, wait: Wait) -> Option<()> {
        let entry = self.queue.write().unwrap().remove(&(wait.pid() as _));
        match entry {
            Some(x) => x.send(Some(wait)).ok().map(|_| ()),
            None => None,
        }
    }
}

/// Returns a reference to the global unique [`ChildQueue`] instance.
fn child_queue() -> &'static ChildQueue {
    CHILD_QUEUE.get_or_init(|| {
        let child_queue = Box::leak(Box::new(ChildQueue::new()));
        _ = child_queue.start();
        child_queue
    })
}

#[must_use]
pub async fn lock() -> impl Drop {
    child_queue().lock.lock().await
}

/// Waits for process termination in nonblocking mode.
///
/// # Errors
/// An `Err(_)` is returned if the underlying OS function failed.
fn wait_nonblocking(pid: Pid) -> std::io::Result<Option<Wait>> {
    let mut status = 0;
    let pid = unsafe { libc::waitpid(pid as _, &mut status, libc::WNOHANG) } as Pid;

    match pid.cmp(&0) {
        cmp::Ordering::Equal => Ok(None),
        cmp::Ordering::Less => Err(std::io::Error::last_os_error()),
        cmp::Ordering::Greater => Ok(Some(Wait::new(pid as _, ExitStatus::from_unix(status)))),
    }
}

/// Converts from [`crate::process::Command`] to [`std::process::Command`].
pub(crate) async fn command_to_std(
    command: &crate::Command,
) -> std::io::Result<std::process::Command> {
    let mut result = std::process::Command::new(&command.program);
    command.args.iter().for_each(|x| {
        result.arg(x);
    });
    if let Some(x) = &command.arg0 {
        result.arg0(x);
    }
    if let Some(x) = &command.env.groups {
        result.groups(x);
    }
    if let Some(x) = command.env.gid {
        result.gid(x);
    }
    if let Some(x) = command.env.uid {
        CommandExt::uid(&mut result, x);
    }
    if let Some(x) = &command.env.root_dir {
        result.root_dir(x);
    }
    if let Some(x) = &command.env.working_dir {
        CommandExt::current_dir(&mut result, x);
    }
    if command.env.clear_vars {
        result.env_clear();
    }
    command.env.vars.iter().for_each(|(k, v)| match v {
        Some(v) => {
            result.env(k, v);
        }
        None => {
            result.env_remove(k);
        }
    });
    result
        .stdout(command.env.stdout.to_std().await?)
        .stderr(command.env.stderr.to_std().await?)
        .stdin(command.env.stdin.to_std().await?);
    if command.env.setsid {
        result.setsid();
    }

    Ok(result)
}

pub(crate) fn command_login(env: &mut CommandEnv, name: &str) -> std::io::Result<()> {
    let (uid, gid, groups_id) = airupfx_env::with_user_by_name(name, |entry| {
        (
            **entry.id(),
            *entry.group_id(),
            entry
                .groups()
                .into_iter()
                .map(|x| **x.id())
                .collect::<Vec<_>>(),
        )
    })
    .ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("user \"{name}\" not found"),
        )
    })?;
    env.uid(uid).gid(gid).groups(groups_id);

    Ok(())
}

pub(crate) async fn spawn(cmd: &crate::Command) -> std::io::Result<Child> {
    Ok(Child::from_std(
        &cmd.env,
        command_to_std(cmd).await?.spawn()?,
    ))
}

pub type WaitError = std::convert::Infallible;
