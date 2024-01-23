//! Signal handling for Unix platforms.

cfg_if::cfg_if! {
    if #[cfg(target_family = "unix")] {
        #[path = "unix.rs"]
        mod sys;
    } else {
        std::compile_error!("This target is not supported by `Airup` yet. Consider opening an issue at https://github.com/sisungo/airup/issues?");
    }
}

pub use sys::*;

/// Ignores all signals in the list. Any errors will be ignored.
pub fn ignore_all<I: IntoIterator<Item = i32>>(signum_list: I) {
    signum_list.into_iter().for_each(|signum| {
        ignore(signum).ok();
    });
}
