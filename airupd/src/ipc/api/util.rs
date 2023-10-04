//! Common utilities for implementing methods.

use crate::{app::airupd, ipc::SessionContext};
use airup_sdk::Error;
use airupfx::{
    config::{system_conf, Security},
    policy::{Action, Actions},
    env::current_uid,
};
use serde::Serialize;

/// Returns `Ok` variant of `Result<serde_json::Value, ApiError>` serializing the given value.
pub fn ok<T: Serialize>(val: T) -> Result<serde_json::Value, Error> {
    Ok(serde_json::to_value(val).unwrap())
}

/// Returns `Ok` variant of `Result<serde_json::Value, ApiError>` with payload `serde_json::json!(null)`.
pub fn ok_null() -> Result<serde_json::Value, Error> {
    Ok(null())
}

/// Returns `serde_json::json!(null)`.
pub fn null() -> serde_json::Value {
    serde_json::json!(null)
}

/// Returns `Ok(())` if given context is permitted to perform the operation, other wise returns `Err(_)`.
pub async fn check_perm(context: &SessionContext, actions: &[Action]) -> Result<(), Error> {
    match system_conf().system.security {
        Security::Disabled => Ok(()),
        Security::Simple => check_perm_simple(context),
        Security::Policy => check_perm_policy(context, actions),
    }
}

fn check_perm_simple(context: &SessionContext) -> Result<(), Error> {
    match &context.uid {
        Some(uid) => match *uid == 0 || *uid == current_uid() {
            true => Ok(()),
            false => Err(Error::permission_denied(["@security_simple"])),
        },
        None => Err(Error::permission_denied(["@channel:credentials"])),
    }
}

fn check_perm_policy(context: &SessionContext, actions: &[Action]) -> Result<(), Error> {
    let actions: Actions = actions.iter().cloned().into();
    match &context.uid {
        Some(uid) => {
            match airupd()
                .storage
                .config
                .policy
                .check(*uid, &actions)
            {
                true => Ok(()),
                false => Err(Error::permission_denied(actions.iter())),
            }
        }
        None => Err(Error::permission_denied(["@channel:credentials"])),
    }
}
