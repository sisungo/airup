//! # `AirupFX` Power Management

#[cfg(target_family = "unix")]
mod unix;

cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        #[path = "macos.rs"]
        mod sys;
    } else if #[cfg(target_os = "linux")] {
        #[path = "linux.rs"]
        mod sys;
    } else if #[cfg(target_os = "freebsd")] {
        #[path = "freebsd.rs"]
        mod freebsd;
    } else {
        #[path = "common.rs"]
        mod sys;
    }
}

use std::convert::Infallible;

/// Interface of power management.
///
/// Methods in this trait are considered to securely reboot the device. These methods should never return (because the host is
/// down) unless the operation failed. To reboot "securely", it should do necessary work before the device is powered down, for
/// example, killing processes, syncing disks, etc.
#[async_trait::async_trait]
pub trait PowerManager: Send + Sync {
    /// Immediately powers the device off.
    ///
    /// # Errors
    /// An `Err(_)` is returned if the underlying OS function failed.
    async fn poweroff(&self) -> std::io::Result<Infallible>;

    /// Immediately reboots the device.
    ///
    /// # Errors
    /// An `Err(_)` is returned if the underlying OS function failed.
    async fn reboot(&self) -> std::io::Result<Infallible>;

    /// Immediately halts the device.
    ///
    /// # Errors
    /// An `Err(_)` is returned if the underlying OS function failed.
    async fn halt(&self) -> std::io::Result<Infallible>;
}

/// A fallback implementation of `AirupFX` power management.
///
/// On this implementation, when power management methods are called, it simply prints "It's now safe to turn off the device."
/// to standard error stream and parks the thread if we are `pid == 1`. Otherwise, it directly exits with code `0`.
#[derive(Default)]
pub struct Fallback;
#[async_trait::async_trait]
impl PowerManager for Fallback {
    async fn poweroff(&self) -> std::io::Result<Infallible> {
        Self::halt_process();
    }

    async fn reboot(&self) -> std::io::Result<Infallible> {
        Self::halt_process();
    }

    async fn halt(&self) -> std::io::Result<Infallible> {
        Self::halt_process();
    }
}
impl Fallback {
    /// Prints "It's now safe to turn off the device." to standard error stream and parks current thread.
    fn halt_process() -> ! {
        if std::process::id() == 1 {
            eprintln!("It's now safe to turn off the device.");
            loop {
                std::thread::park();
            }
        } else {
            std::process::exit(0);
        }
    }
}

/// Returns a reference to the global unique [`PowerManager`] instance.
///
/// If the process is `pid == 1`, the platform power manager is used, otherwise the fallback power manager [`Fallback`] is
/// always returned.
#[must_use]
pub fn power_manager() -> &'static dyn PowerManager {
    if std::process::id() == 1 {
        sys::power_manager()
    } else {
        &Fallback
    }
}
