//! APIs that provides information about Airup and the system.

use super::{Method, MethodFuture};
use crate::ipc::{api::util::ok, SessionContext};
use airupfx::ipc::mapi::Request;
use std::{collections::HashMap, hash::BuildHasher, sync::Arc};

pub fn init<H: BuildHasher>(methods: &mut HashMap<&'static str, Method, H>) {
    methods.insert("info.airupd.version", airupd_version);
    methods.insert("info.airupfx.version", airupfx_version);
    methods.insert("info.airupfx.build_manifest", airupfx_build_manifest);
}

fn airupd_version(_: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async { ok(env!("CARGO_PKG_VERSION")) })
}

fn airupfx_version(_: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async { ok(airupfx::VERSION) })
}

fn airupfx_build_manifest(_: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async { ok(airupfx::config::build_manifest()) })
}
