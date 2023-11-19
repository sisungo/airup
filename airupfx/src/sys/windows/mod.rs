//! Platform-specific functions for Microsoft Windows.

std::compile_error!("Support of Microsoft Windows is a work-in-progress");

pub mod env;
pub mod fs;
pub mod power;
pub mod process;
pub mod signal;

pub fn power_manager() -> &'static dyn crate::power::PowerManager {
    &power::Windows
}

pub fn init() {}
