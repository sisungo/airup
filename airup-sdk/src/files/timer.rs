//! # Airup Timer File Format
//! This module contains [`Timer`], the main file format of an Airup service and its combinations.

use super::{Named, ReadError, Validate};
use serde::{Deserialize, Serialize};

/// An Airup timer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Timer {}
impl Validate for Timer {
    fn validate(&self) -> Result<(), ReadError> {
        Ok(())
    }
}
impl Named for Timer {
    fn set_name(&mut self, _: String) {}
}
