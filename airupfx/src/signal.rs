pub use crate::sys::signal::*;

/// Ignores a signal.
///
/// ## Errors
/// An `Err(_)` is returned if the underlying OS function failed.
pub fn ignore(signum: i32) -> anyhow::Result<()> {
    signal(signum, |_| async {})
}

/// Ignores all signals in the list. Any errors will be ignored.
pub fn ignore_all<I: IntoIterator<Item = i32>>(signum_list: I) {
    signum_list.into_iter().for_each(|signum| {
        ignore(signum).ok();
    });
}
