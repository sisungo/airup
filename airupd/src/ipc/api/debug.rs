//! APIs that provides Airup debugging utilities.

use super::{
    util::{check_perm, ok, ok_null},
    Method, MethodFuture, SessionContext,
};
use crate::app::airupd;
use airup_sdk::{
    error::ApiError,
    ipc::{Request, Response},
};
use airupfx::policy::Action;
use std::{collections::HashMap, hash::BuildHasher, sync::Arc};

pub fn init<H: BuildHasher>(methods: &mut HashMap<&'static str, Method, H>) {
    methods.insert("debug.echo_raw", echo_raw);
    methods.insert("debug.dump", dump);
    methods.insert("debug.exit", exit);
    methods.insert("debug.reload_image", reload_image);
}

fn echo_raw(_: Arc<SessionContext>, x: Request) -> MethodFuture {
    Box::pin(async {
        x.extract_params::<Response>()
            .unwrap_or_else(|x| Response::Err(ApiError::invalid_params(x)))
            .into_result()
    })
}

fn dump(context: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async move {
        check_perm(&context, &[Action::Dump])?;
        ok(format!("{:#?}", airupd()))
    })
}

fn exit(context: Arc<SessionContext>, x: Request) -> MethodFuture {
    Box::pin(async move {
        check_perm(&context, &[Action::Power])?;
        airupd()
            .lifetime
            .exit(x.extract_params().unwrap_or_default());
        ok_null()
    })
}

fn reload_image(context: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async move {
        check_perm(&context, &[Action::Power])?;
        airupd().lifetime.reload_image();
        ok_null()
    })
}
