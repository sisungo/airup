//! Platform-specific functions for Unix platforms.

#![allow(unused)]

pub mod env;
pub mod fs;
pub mod process;
pub mod signal;
pub mod std_port;

pub fn power_manager() -> &'static dyn crate::power::PowerManager {
    &crate::power::Fallback
}

pub fn init() {
    process::init();
}
