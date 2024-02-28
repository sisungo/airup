//! # Airup Timer File Format
//! This module contains [`Timer`], the main file format of an Airup service and its combinations.

use super::{Named, ReadError, Validate};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

/// An Airup timer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Timer {
    pub timer: Metadata,
    pub exec: Exec,
}
impl Validate for Timer {
    fn validate(&self) -> Result<(), ReadError> {
        let clock_normal_has_period = matches!(
            self.timer.clock,
            Clock::OnLoad | Clock::Persistent | Clock::SystemBoot
        ) && self.timer.period.is_some();
        let clock_calendar_no_period =
            matches!(self.timer.clock, Clock::Calendar) && self.timer.period.is_none();

        if !clock_normal_has_period {
            return Err(ReadError::from(
                "normal clocks require a `period` to be provided, but it isn't provided",
            ));
        }
        if !clock_calendar_no_period {
            return Err(ReadError::from(
                "`calendar` clocks require `period` to be not provided, but it's provided",
            ));
        }

        Ok(())
    }
}
impl Named for Timer {
    fn set_name(&mut self, _: String) {}
}

/// Metadata of an Airup timer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Metadata {
    /// Type of clock that the timer uses.
    pub clock: Clock,

    /// Period of the timer, in milliseconds.
    pub period: Option<NonZeroU32>,
}

/// Represents to a clock.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Clock {
    /// Represents to a clock created when the timer is loaded by the event source.
    #[default]
    OnLoad,

    /// Represents to a persistent clock, which is created on `airup-eventsourced`'s first run.
    Persistent,

    /// Represents to a clock created on system boot.
    SystemBoot,

    /// Represents to a clock which rings when the specified calendar condition was reached.
    Calendar,
}

/// Executation of a timer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Exec {
    /// The command to execute when the time arrived.
    pub command: String,
}
