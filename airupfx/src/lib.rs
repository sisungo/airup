//! # AirupFX
//! AirupFX is the unified framework for Airup developing.

#[cfg(feature = "files")]
pub mod files;

#[cfg(feature = "fs")]
pub mod fs;

#[cfg(feature = "log")]
pub mod log;

#[cfg(feature = "config")]
pub mod config;

#[cfg(feature = "ipc")]
pub mod ipc;

#[cfg(feature = "sdk")]
pub mod sdk;

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

pub mod util;

pub mod sync;

pub mod prelude;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
