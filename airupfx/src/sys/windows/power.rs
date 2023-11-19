//! Microsoft Windows Power Management.

use crate::power::PowerManager;
use std::convert::Infallible;

#[derive(Default)]
pub struct Windows;
impl PowerManager for Windows {
    fn poweroff(&self) -> std::io::Result<Infallible> {
        unimplemented!()
    }

    fn reboot(&self) -> std::io::Result<Infallible> {
        unimplemented!()
    }

    fn halt(&self) -> std::io::Result<Infallible> {
        unimplemented!()
    }
}
