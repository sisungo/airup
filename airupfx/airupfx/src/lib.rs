//! # AirupFX
//! Base OS support library of Airup. This is internal to the `airup` project and is NOT subjected to be published as a part of
//! the Airup SDK.
//!
//! Since Airup v0.5.0, AirupFX version is no longer synced with other components.

pub mod ace;
pub mod log;
pub mod prelude;
pub mod signal;
pub mod sys;
pub mod time;
pub mod util;

pub use airupfx_env as env;
pub use airupfx_fs as fs;
pub use airupfx_io as io;
pub use airupfx_power as power;
pub use airupfx_process as process;
