//! Common utilities for implementing methods.

use crate::{app::airupd, ipc::SessionContext};
use airupfx::{
    config::{system_conf, Security},
    policy::{Action, Actions},
    users::current_uid,
};
use airup_sdk::Error;
use serde::Serialize;

/// Returns `Ok(())` if given context is permitted to perform the operation, other wise returns `Err(_)`.
pub async fn check_perm(context: &SessionContext, actions: &[Action]) -> Result<(), Error> {
    let no_credentials = || Err(Error::permission_denied(["@channel:credentials"]));
    match system_conf().system.security {
        Security::Disabled => Ok(()),
        Security::Simple => match &context.uid {
            Some(uid) => match *uid == current_uid() || *uid == 0 {
                true => Ok(()),
                false => Err(Error::permission_denied(["@security_simple"])),
            },
            None => no_credentials(),
        },
        Security::Policy => {
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
                        false => Err(Error::permission_denied(actions.iter())),
                    }
                }
                None => no_credentials(),
            }
        }
    }
}

/// Returns `Ok` variant of `Result<serde_json::Value, ApiError>` serializing the given value.
#[inline]
pub fn ok<T: Serialize>(val: T) -> Result<serde_json::Value, Error> {
    Ok(serde_json::to_value(val).unwrap())
}

/// Returns `Ok` variant of `Result<serde_json::Value, ApiError>` with payload `serde_json::json!(null)`.
#[inline]
pub fn ok_null() -> Result<serde_json::Value, Error> {
    Ok(null())
}

/// Returns `serde_json::json!(null)`.
#[inline]
pub fn null() -> serde_json::Value {
    serde_json::json!(null)
}
