pub mod power;
pub use super::unix::env;
pub use super::unix::process;
pub use super::unix::signal;

pub fn power_manager() -> &'static dyn crate::power::PowerManager {
    &power::FreeBsd
}

pub fn init() {
    super::unix::init();
}
