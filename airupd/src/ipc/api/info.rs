//! APIs that provides information about Airup and the system.

use super::{Method, MethodFuture};
use crate::ipc::SessionContext;
use airup_sdk::{build::BuildManifest, ipc::Request, Error};
use std::{collections::HashMap, hash::BuildHasher, sync::Arc};

pub fn init<H: BuildHasher>(methods: &mut HashMap<&'static str, Method, H>) {
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
