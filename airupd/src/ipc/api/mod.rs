//! # Airup IPC API - Implementation

mod debug;
mod info;
mod system;

use ahash::AHashMap;
use airup_sdk::{
    ipc::{Request, Response},
    Error,
};
use airupfx::prelude::*;

/// The Airup IPC API (implementation) manager.
#[derive(Debug)]
pub struct Manager {
    methods: AHashMap<&'static str, Method>,
}
impl Manager {
    /// Creates a new `Manager` instance.
    pub fn new() -> Self {
        let mut object = Self {
            methods: AHashMap::with_capacity(32),
        };
        object.init();
        object
    }

    /// Initializes the `Manager` instance.
    pub fn init(&mut self) {
        info::init(&mut self.methods);
        debug::init(&mut self.methods);
        system::init(&mut self.methods);
    }

    /// Invokes a method by the given request.
    pub(super) async fn invoke(&self, req: Request) -> Response {
        let method = self.methods.get(&req.method[..]).copied();
        match method {
            Some(method) => Response::new(method(req).await),
            None => Response::Err(Error::NoSuchMethod),
        }
    }
}
impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents to an IPC method.
pub(super) type Method = fn(Request) -> MethodFuture;

/// Represents to future type of an IPC method.
pub type MethodFuture = BoxFuture<'static, Result<serde_json::Value, Error>>;

#[macro_export]
macro_rules! ipc_methods {
    ($prefix:ident, [$($n:ident),*,]) => {
        [
            $((concat!(stringify!($prefix), ".", stringify!($n)), $n as Method)),*
        ]
    };
}
