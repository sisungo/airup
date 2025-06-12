//! APIs that provides information about Airup and the system.

use super::MethodFuture;
use crate::rpc::route::Router;
use airup_sdk::{Error, build::BuildManifest};

pub fn router() -> Router {
    Router::new()
        .route("version", version)
        .route("build_manifest", build_manifest)
}

#[airupfx::macros::api]
async fn version() -> Result<&'static str, Error> {
    Ok(env!("CARGO_PKG_VERSION"))
}

#[airupfx::macros::api]
async fn build_manifest() -> Result<&'static BuildManifest, Error> {
    Ok(airup_sdk::build::manifest())
}
