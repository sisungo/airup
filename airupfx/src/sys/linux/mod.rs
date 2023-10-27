pub mod power;
pub mod process;
pub use super::unix::env;
pub use super::unix::signal;

pub fn power_manager() -> &'static dyn crate::power::PowerManager {
    &power::Linux
}

pub fn init() {
    super::unix::init();
    if *crate::process::ID != 1 {
        process::become_subreaper().ok();
    }
}
