//! Platform-specific functions for Unix platforms.

#![allow(unused)]

pub mod signal;

pub fn power_manager() -> &'static dyn crate::power::PowerManager {
    &crate::power::Fallback
}
