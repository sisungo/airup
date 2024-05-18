//! # AirupFX
//! Base OS support library of Airup. This is internal to the `airup` project and is NOT subjected to be published as a part of
//! the Airup SDK.
//!
//! Since Airup v0.5.0, AirupFX version is no longer synced with other components.

pub mod prelude;
pub mod util;

pub use airupfx_env as env;
pub use airupfx_extensions as extensions;
pub use airupfx_fs as fs;
pub use airupfx_io as io;
pub use airupfx_isolator as isolator;
pub use airupfx_macros as macros;
pub use airupfx_power as power;
pub use airupfx_process as process;
pub use airupfx_signal as signal;
pub use airupfx_time as time;

pub async fn init() {
    #[cfg(feature = "selinux")]
    {
        if process::as_pid1() && env::take_var("AIRUP_TEMP_SELINUX_INITIALIZED").is_err() {
            _ = selinux::policy::load_initial();
            std::env::set_var("AIRUP_TEMP_SELINUX_INITIALIZED", "1");
            _ = process::reload_image();
        }
    }
}
