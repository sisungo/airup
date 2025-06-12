//! APIs that provides Airup debugging utilities.

use super::MethodFuture;
use crate::{app::airupd, rpc::route::Router};
use airup_sdk::{
    error::ApiError,
    rpc::{Request, Response},
};

pub fn router() -> Router {
    Router::new()
        .route("echo_raw", echo_raw)
        .route("dump", dump)
        .route("exit", exit)
        .route("is_forking_supervisable", is_forking_supervisable)
}

fn echo_raw(x: Request) -> MethodFuture {
    Box::pin(async {
        x.extract_params::<Response>()
            .unwrap_or_else(|x| Response::Err(ApiError::invalid_params(x)))
            .into_result()
    })
}

#[airupfx::macros::api]
async fn dump() -> Result<String, ApiError> {
    Ok(format!("{:#?}", airupd()))
}

#[airupfx::macros::api]
async fn exit(code: i32) -> Result<(), ApiError> {
    airupd().lifetime.exit(code);
    Ok(())
}

#[airupfx::macros::api]
async fn is_forking_supervisable() -> Result<bool, ApiError> {
    Ok(airupfx::process::is_forking_supervisable())
}
