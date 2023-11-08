//! Microsoft Windows Power Management.

use std::convert::Infallible;
use crate::power::PowerManager;

#[derive(Default)]
pub struct Windows;
impl PowerManager for Windows {
    fn poweroff(&self) -> std::io::Result<Infallible> {
        todo!()
    }

    fn reboot(&self) -> std::io::Result<Infallible> {
        todo!()
    }

    fn halt(&self) -> std::io::Result<Infallible> {
        todo!()
    }
}