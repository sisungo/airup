use crate::PowerManager;

pub fn power_manager() -> &'static dyn PowerManager {
    &crate::Fallback
}
