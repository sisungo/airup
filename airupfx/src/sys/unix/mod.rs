//! Platform-specific functions for Unix platforms.

pub mod env;
pub mod fs;
pub mod process;
pub mod signal;
pub mod std_port;

#[allow(unused)]
pub fn power_manager() -> &'static dyn crate::power::PowerManager {
    &crate::power::Fallback
}

#[allow(unused)]
pub fn init() {
    process::init();
}
