//! # Airup Timer File Format
//! This module contains [`Timer`], the main file format of an Airup service and its combinations.

use serde::{Deserialize, Serialize};

/// An Airup timer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Timer {}
