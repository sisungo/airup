#![allow(unused)]

use super::{Security, Security::*};
use std::{path::Path, sync::OnceLock};

macro_rules! map {
    (@$key:literal : $val:literal) => {
        ($key, Some($val))
    };
    (@$key:literal : null) => {
        ($key, None)
    };
    ($($key:literal : $val:literal),* ,) => (
        map!($($key => $val),*)
    );
    ($($key:literal : $val:tt),*) => ({
        &[$(map!(@$key => $val))*]
    });
    ({ $($key:literal : $val:tt),* }) => {
        map!($($key => $val),*)
    };
    ({ $($key:literal : $val:tt),* ,}) => {
        map!($($key => $val),*)
    };
}
macro_rules! build_manifest {
    (@$result:expr, os_name : $val:literal) => {
        $result.os_name = $val;
    };
    (@$result:expr, config_dir : $val:literal) => {
        $result.config_dir = ::std::path::Path::new($val);
    };
    (@$result:expr, service_dir : $val:literal) => {
        $result.service_dir = ::std::path::Path::new($val);
    };
    (@$result:expr, milestone_dir : $val:literal) => {
        $result.milestone_dir = ::std::path::Path::new($val);
    };
    (@$result:expr, runtime_dir : $val:literal) => {
        $result.runtime_dir = ::std::path::Path::new($val);
    };
    (@$result:expr, env_vars : $val:tt) => {
        $result.env_vars = map!($val);
    };
    (@$result:expr, security : $val:expr) => {
        $result.security = $val;
    };
    ($($key:ident : $val:tt),* ,) => {
        build_manifest!($($key : $val),*)
    };
    ($($key:ident : $val:tt),*) => {
        {
            let mut result = BuildManifest::default();
            $(
                build_manifest!(@result, $key : $val);
            )*
            result
        }
    };
}

/// Represents to `build_manifest.json`.
#[derive(Debug, Clone)]
pub struct BuildManifest {
    /// Name of the running operating system.
    pub os_name: &'static str,

    /// Path of Airup's system-wide config directory, e.g. `/etc/airup`.
    pub config_dir: &'static Path,

    /// Path of Airup's system-wide service directory, e.g. `/etc/airup/services`.
    pub service_dir: &'static Path,

    /// Path of Airup's system-wide milestone directory, e.g. `/etc/airup/milestones`.
    pub milestone_dir: &'static Path,

    /// Path of Airup's system-wide runtime directory, e.g. `/run/airup`.
    pub runtime_dir: &'static Path,

    /// Table of initial environment variables.
    pub env_vars: &'static [(&'static str, Option<&'static str>)],

    pub security: Security,
}
impl BuildManifest {
    pub fn parse() -> Self {
        include!("../../../build_manifest.rs")
    }

    #[inline]
    pub fn get() -> &'static Self {
        static MANIFEST: OnceLock<BuildManifest> = OnceLock::new();

        MANIFEST.get_or_init(Self::parse)
    }
}
impl Default for BuildManifest {
    fn default() -> Self {
        Self {
            os_name: "\x1b[36;4mAirup\x1b[0m",
            config_dir: Path::new(""),
            service_dir: Path::new(""),
            milestone_dir: Path::new(""),
            runtime_dir: Path::new(""),
            env_vars: &[],
            security: Security::Policy,
        }
    }
}
