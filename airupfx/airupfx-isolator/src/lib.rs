cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        #[path = "linux.rs"]
        mod sys;
    } else {
        #[path = "fallback.rs"]
        mod sys;
    }
}

#[derive(Debug)]
pub struct Controller(sys::Controller);

#[derive(Debug)]
pub struct Realm(sys::Realm);
