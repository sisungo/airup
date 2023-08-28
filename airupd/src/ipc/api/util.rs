//! Common utilities for implementing methods.

use crate::{app::airupd, ipc::SessionContext};
use airupfx::{
    ipc::mapi::ApiError,
    policy::{Action, Actions},
};
use serde::Serialize;

/// Returns `Ok(())` if given context is permitted to perform the operation, other wise returns `Err(_)`.
pub async fn check_perm(context: &SessionContext, actions: &[Action]) -> Result<(), ApiError> {
    let actions: Actions = actions.iter().cloned().into();
    match &context.uid {
        Some(uid) => {
            match airupd()
                .storage
                .config
                .policy
                .get()
                .await
                .check(*uid, &actions)
            {
                true => Ok(()),
                false => Err(ApiError::permission_denied(actions.iter())),
            }
        }
        None => Err(ApiError::permission_denied(["channel:credentials"])),
    }
}

/// Returns `Ok` variant of `Result<serde_json::Value, ApiError>` serializing the given value.
#[inline]
pub fn ok<T: Serialize>(val: T) -> Result<serde_json::Value, ApiError> {
    Ok(serde_json::to_value(val).unwrap())
}

/// Returns `Ok` variant of `Result<serde_json::Value, ApiError>` with payload `serde_json::json!(null)`.
#[inline]
pub fn ok_null() -> Result<serde_json::Value, ApiError> {
    Ok(null())
}

/// Returns `serde_json::json!(null)`.
#[inline]
pub fn null() -> serde_json::Value {
    serde_json::json!(null)
}
