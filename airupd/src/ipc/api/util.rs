//! Common utilities for implementing methods.

use airup_sdk::Error;
use serde::Serialize;

/// Returns `Ok` variant of `Result<serde_json::Value, ApiError>` serializing the given value.
pub fn ok<T: Serialize>(val: T) -> Result<serde_json::Value, Error> {
    Ok(serde_json::to_value(val)
        .expect("IPC methods should return a value that can be serialized into JSON"))
}

/// Returns `Ok` variant of `Result<serde_json::Value, ApiError>` with payload `serde_json::json!(null)`.
pub fn ok_null() -> Result<serde_json::Value, Error> {
    Ok(null())
}

/// Returns `serde_json::json!(null)`.
pub fn null() -> serde_json::Value {
    serde_json::json!(null)
}
