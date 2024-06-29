//! APIs that provides Airup debugging utilities.

use super::{Method, MethodFuture};
use crate::app::airupd;
use airup_sdk::{
    error::ApiError,
    rpc::{Request, Response},
};
use std::{collections::HashMap, hash::BuildHasher};

pub(super) fn init<H: BuildHasher>(methods: &mut HashMap<&'static str, Method, H>) {
    crate::ipc_methods!(debug, [echo_raw, dump, exit, is_forking_supervisable,])
        .iter()
        .for_each(|(k, v)| {
            methods.insert(k, *v);
        });
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
