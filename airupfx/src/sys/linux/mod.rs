//! Platform-specific functions for the Linux kernel.

pub mod power;
pub mod process;
pub use super::unix::env;
pub use super::unix::fs;
pub use super::unix::signal;

pub fn power_manager() -> &'static dyn crate::power::PowerManager {
    &power::Linux
}

pub fn init() {
    super::unix::init();

    // Linux supports the feature of child subreapers, which allows us to reap grandchild processes as we are not `pid == 1`.
    if std::process::id() != 1 {
        process::become_subreaper().ok();
    }
}
