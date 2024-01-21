//! APIs that provides Airup debugging utilities.

use super::{
    util::{ok, ok_null},
    Method, MethodFuture, SessionContext,
};
use crate::app::airupd;
use airup_sdk::{
    error::ApiError,
    ipc::{Request, Response},
};
use std::{collections::HashMap, hash::BuildHasher, sync::Arc};

pub fn init<H: BuildHasher>(methods: &mut HashMap<&'static str, Method, H>) {
    crate::ipc_methods!(debug, [echo_raw, dump, exit, is_forking_supervisable,])
        .iter()
        .for_each(|(k, v)| {
            methods.insert(k, *v);
        });
}

fn echo_raw(_: Arc<SessionContext>, x: Request) -> MethodFuture {
    Box::pin(async {
        x.extract_params::<Response>()
            .unwrap_or_else(|x| Response::Err(ApiError::invalid_params(x)))
            .into_result()
    })
}

fn dump(_: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async move { ok(format!("{:#?}", airupd())) })
}

fn exit(_: Arc<SessionContext>, x: Request) -> MethodFuture {
    Box::pin(async move {
        airupd()
            .lifetime
            .exit(x.extract_params().unwrap_or_default());
        ok_null()
    })
}

fn is_forking_supervisable(_: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async move { ok(airupfx::process::is_forking_supervisable()) })
}
