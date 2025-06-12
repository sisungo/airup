//! # Airup IPC API - Implementation

mod debug;
mod info;
pub mod session;
mod system;

use crate::rpc::route::Router;
use airup_sdk::{Error, rpc::Request};
use airupfx::prelude::*;

pub fn root_router() -> Router {
    Router::new()
        .nest("system", system::router())
        .nest("debug", debug::router())
        .nest("info", info::router())
}

/// Represents to an IPC method.
pub(super) type Method = fn(Request) -> MethodFuture;

/// Represents to future type of an IPC method.
pub type MethodFuture = BoxFuture<'static, Result<ciborium::Value, Error>>;
