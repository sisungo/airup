//! # AirupFX
//! AirupFX is the unified framework for Airup developing.

#[cfg(feature = "fs")]
pub mod fs;

#[cfg(feature = "log")]
pub mod log;

#[cfg(feature = "config")]
pub mod config;

#[cfg(feature = "policy")]
pub mod policy;

#[cfg(feature = "users")]
pub mod users;

#[cfg(feature = "process")]
pub mod process;

#[cfg(feature = "env")]
pub mod env;

#[cfg(feature = "power")]
pub mod power;

#[cfg(feature = "ace")]
pub mod ace;

#[cfg(feature = "time")]
pub mod time;

#[cfg(feature = "signal")]
pub mod signal;

pub mod util;

pub mod std_port;

pub mod sys;

pub mod collections;

pub mod sync;

pub mod prelude;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
