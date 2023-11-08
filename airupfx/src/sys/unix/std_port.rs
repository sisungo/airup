//! Port of Unix-specific unstable `std` features in stable Rust.
//!
//! These will be deleted when the features gets stablized, or moved if they are determined to remove.

use std::os::unix::process::CommandExt as _;

/// Port of unstable feature `#![feature(process_setsid)]` and `#![feature(setgroups)]` to stable Rust.
///
/// This will be deleted when the feature is stablized.
pub trait CommandExt {
    fn setsid(&mut self) -> &mut Self;
    fn groups(&mut self, groups: &[libc::gid_t]) -> &mut Self;
}
impl CommandExt for std::process::Command {
    fn setsid(&mut self) -> &mut Self {
        unsafe { self.pre_exec(|| crate::sys::env::setsid().map(|_| ())) }
    }

    fn groups(&mut self, groups: &[libc::gid_t]) -> &mut Self {
        let groups: Vec<_> = groups.iter().map(|x| *x).collect();
        unsafe { self.pre_exec(move || crate::sys::env::setgroups(&groups[..])) }
    }
}
