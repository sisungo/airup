//! # `AirupFX` Power Management

use std::convert::Infallible;

/// Interface of power management.
pub trait PowerManager: Send + Sync {
    /// Immediately powers the device off.
    ///
    /// # Errors
    /// An `Err(_)` is returned if the underlying OS function failed.
    fn poweroff(&self) -> std::io::Result<Infallible>;

    /// Immediately reboots the device.
    ///
    /// # Errors
    /// An `Err(_)` is returned if the underlying OS function failed.
    fn reboot(&self) -> std::io::Result<Infallible>;

    /// Immediately halts the device.
    ///
    /// # Errors
    /// An `Err(_)` is returned if the underlying OS function failed.
    fn halt(&self) -> std::io::Result<Infallible>;
}

/// A fallback implementation of `AirupFX` power management.
///
/// On this implementation, when power management methods are called, it simply prints "It's now safe to turn off the device."
/// to standard error stream and parks the thread.
#[derive(Default)]
pub struct Fallback;
impl Fallback {
    pub const GLOBAL: &'static Self = &Self;
}
impl PowerManager for Fallback {
    fn poweroff(&self) -> std::io::Result<Infallible> {
        Self::halt_process();
    }

    fn reboot(&self) -> std::io::Result<Infallible> {
        Self::halt_process();
    }

    fn halt(&self) -> std::io::Result<Infallible> {
        Self::halt_process();
    }
}
impl Fallback {
    /// Prints "It's now safe to turn off the device." to standard error stream and parks current thread.
    fn halt_process() -> ! {
        if *crate::process::ID == 1 {
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
#[must_use]
pub fn power_manager() -> &'static dyn PowerManager {
    if *crate::process::ID == 1 {
        crate::sys::power_manager()
    } else {
        Fallback::GLOBAL
    }
}
