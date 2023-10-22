pub mod power;
pub use super::unix::process;
pub use super::unix::signal;
pub use super::unix::env;

pub fn power_manager() -> &'static dyn crate::power::PowerManager {
    &power::Linux
}

pub fn init() {
    super::unix::init();
}