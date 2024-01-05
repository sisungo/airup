//! APIs that provides information about Airup and the system.

use super::{Method, MethodFuture};
use crate::ipc::{api::util::ok, SessionContext};
use airup_sdk::ipc::Request;
use std::{collections::HashMap, hash::BuildHasher, sync::Arc};

pub fn init<H: BuildHasher>(methods: &mut HashMap<&'static str, Method, H>) {
    crate::ipc_methods!(info, [version, build_manifest,])
        .iter()
        .for_each(|(k, v)| {
            methods.insert(k, *v);
        });
    methods.insert("info.version", version);
    methods.insert("info.build_manifest", build_manifest);
}

fn version(_: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async { ok(env!("CARGO_PKG_VERSION")) })
}

fn build_manifest(_: Arc<SessionContext>, _: Request) -> MethodFuture {
    Box::pin(async { ok(airup_sdk::build::manifest()) })
}
