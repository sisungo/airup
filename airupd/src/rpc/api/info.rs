//! APIs that provides information about Airup and the system.

use super::{Method, MethodFuture};
use airup_sdk::{Error, build::BuildManifest};
use std::{collections::HashMap, hash::BuildHasher};

pub(super) fn init<H: BuildHasher>(methods: &mut HashMap<&'static str, Method, H>) {
    crate::ipc_methods!(info, [version, build_manifest,])
        .iter()
        .for_each(|(k, v)| {
            methods.insert(k, *v);
        });
}

#[airupfx::macros::api]
async fn version() -> Result<&'static str, Error> {
    Ok(env!("CARGO_PKG_VERSION"))
}

#[airupfx::macros::api]
async fn build_manifest() -> Result<&'static BuildManifest, Error> {
    Ok(airup_sdk::build::manifest())
}
