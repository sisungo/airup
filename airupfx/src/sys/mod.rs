#[cfg(target_family = "unix")]
pub mod unix;

#[cfg(target_family = "unix")]
pub use unix::*;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(any(
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "dragonfly"
))]
pub mod bsd;

#[cfg(any(
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "dragonfly"
))]
pub use bsd::*;

/// Returns a reference to the global default [crate::power::PowerManager] instance.
#[allow(unreachable_code)]
#[cfg(feature = "power")]
pub fn power_manager() -> &'static dyn crate::power::PowerManager {
    #[cfg(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    ))]
    return bsd::power::Bsd::GLOBAL;

    #[cfg(target_os = "linux")]
    return linux::power::Linux::GLOBAL;

    crate::power::Fallback::GLOBAL
}