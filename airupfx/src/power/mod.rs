//! # AirupFX Power Management

#[cfg(target_os = "linux")]
mod linux;

#[cfg(any(
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "dragonfly"
))]
mod bsd;

use std::{convert::Infallible, sync::OnceLock};

static POWER_MANAGER: OnceLock<Box<dyn PowerManager>> = OnceLock::new();

/// Interface of power management.
pub trait PowerManager: Send + Sync {
    /// Immediately shuts the device down.
    fn shutdown(&self) -> std::io::Result<Infallible>;

    /// Immediately reboots the device.
    fn reboot(&self) -> std::io::Result<Infallible>;

    /// Immediately halts the device.
    fn halt(&self) -> std::io::Result<Infallible>;
}

/// A fallback implementation of AirupFX power management.
///
/// On this implementation, when a power management method is called, it simply prints "It's now safe to turn off the device."
/// to `stderr` and parks the thread.
#[derive(Default)]
pub struct Fallback;
impl PowerManager for Fallback {
    fn shutdown(&self) -> std::io::Result<Infallible> {
        self.halt_process();
    }

    fn reboot(&self) -> std::io::Result<Infallible> {
        self.halt_process();
    }

    fn halt(&self) -> std::io::Result<Infallible> {
        self.halt_process();
    }
}
impl Fallback {
    /// Prints "It's now safe to turn off the device." to `stderr` and parks current thread.
    #[inline]
    fn halt_process(&self) -> ! {
        eprintln!("It's now safe to turn off the device.");
        loop {
            std::thread::park();
        }
    }
}

/// Returns a reference to the global unique [PowerManager] instance.
pub fn power_manager() -> &'static dyn PowerManager {
    &**POWER_MANAGER.get_or_init(default_power_manager)
}

/// Returns the default [PowerManager] object of current platform.
#[allow(unreachable_code)]
pub fn default_power_manager() -> Box<dyn PowerManager> {
    #[cfg(target_os = "linux")]
    return Box::<linux::Linux>::default();

    #[cfg(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    ))]
    return Box::<bsd::Bsd>::default();

    Box::<Fallback>::default()
}
