//! Platform-specific functions for Microsoft Windows.

std::compile_error!("Support of Microsoft Windows is work-in-progress!");

pub mod process;
pub mod signal;
pub mod env;
pub mod power;
pub mod fs;

pub fn power_manager() -> &'static dyn crate::power::PowerManager {
    &power::Windows
}

pub fn init() {
    todo!()
}