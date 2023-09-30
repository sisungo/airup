//! Port of unstable `std` features in stable Rust.
//!
//! These will be deleted when the features gets stablized.

use std::os::unix::process::CommandExt as _;
use crate::env::Gid;

/// Port of unstable feature `#![feature(result_option_inspect)]` to stable Rust.
///
/// This will be deleted when the feature is stablized.
pub trait ResultExt<T, E> {
    fn inspect_err<F: FnOnce(&E)>(self, op: F) -> Self;
}
impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn inspect_err<F: FnOnce(&E)>(self, op: F) -> Self {
        self.map_err(|e| {
            op(&e);
            e
        })
    }
}

/// Port of unstable feature `#![feature(result_option_inspect)]` to stable Rust.
///
/// This will be deleted when the feature is stablized.
pub trait OptionExt<T> {
    fn inspect_none<F: FnOnce()>(self, op: F) -> Self;
}
impl<T> OptionExt<T> for Option<T> {
    fn inspect_none<F: FnOnce()>(self, op: F) -> Self {
        if self.is_none() {
            op()
        }

        self
    }
}

/// Port of unstable feature `#![feature(process_setsid)]` and `#![feature(setgroups)]` to stable Rust.
/// 
/// This will be deleted when the feature is stablized.
pub trait CommandExt {
    fn setsid(&mut self) -> &mut Self;
    fn groups(&mut self, groups: &[Gid]) -> &mut Self;
}
impl CommandExt for std::process::Command {
    fn setsid(&mut self) -> &mut Self {
        unsafe {
            self.pre_exec(|| crate::sys::env::setsid().map(|_| ()))
        }
    }

    fn groups(&mut self, groups: &[Gid]) -> &mut Self {
        let groups: Vec<_> = groups.iter().map(|x| *x as _).collect();
        unsafe {
            self.pre_exec(move || crate::sys::env::setgroups(&groups))
        }
    }
}