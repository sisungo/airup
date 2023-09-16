//! # Airup IPC API - Implementation

mod debug;
mod info;
mod system;
pub mod util;

use super::SessionContext;
use ahash::AHashMap;
use airup_sdk::{
    error::ApiError,
    ipc::{Request, Response},
};
use airupfx::prelude::*;
use std::sync::{Arc, RwLock};

/// The Airup IPC API (implementation) manager.
#[derive(Debug)]
pub struct Manager {
    methods: RwLock<AHashMap<&'static str, Method>>,
}
impl Manager {
    /// Creates a new `Manager` instance.
    pub fn new() -> Self {
        let object = Self {
            methods: RwLock::new(AHashMap::with_capacity(128)),
        };
        object.init();
        object
    }

    /// Initializes the `Manager` instance.
    pub fn init(&self) {
        let mut lock = self.methods.write().unwrap();
        info::init(&mut lock);
        debug::init(&mut lock);
        system::init(&mut lock);
    }

    /// Invokes a method by the given request.
    pub async fn invoke(&self, context: Arc<SessionContext>, req: Request) -> Response {
        let method = self.methods.read().unwrap().get(&req.method[..]).copied();
        match method {
            Some(method) => Response::new(method(context, req).await),
            None => Response::Err(ApiError::NoSuchMethod),
        }
    }
}
impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents to an IPC method.
pub type Method = fn(Arc<SessionContext>, Request) -> MethodFuture;

/// Represents to future type of an IPC method.
pub type MethodFuture = BoxFuture<'static, Result<serde_json::Value, ApiError>>;
