use once_cell::sync::Lazy;
use serde::de::Error;
use std::sync::RwLock;
use sysinfo::{SystemExt, Uid, User, UserExt, Gid};

static USERS: Lazy<RwLock<sysinfo::System>> = Lazy::new(|| {
    let mut instance = sysinfo::System::new();
    instance.refresh_users_list();
    instance.into()
});

/// Refreshes users database.
pub async fn refresh() {
    USERS.write().unwrap().refresh_users_list()
}

/// Finds a user entry by UID.
pub async fn with_user_by_id<F: FnOnce(&User) -> T, T>(uid: &Uid, f: F) -> Option<T> {
    USERS.read().unwrap().get_user_by_id(uid).map(f)
}

/// Finds a user entry by username.
pub async fn with_user_by_name<F: FnOnce(&User) -> T, T>(name: &String, f: F) -> Option<T> {
    USERS
        .read()
        .unwrap()
        .users()
        .iter()
        .find(|u| u.name() == name)
        .map(f)
}

/// Returns the user entry of current user.
pub async fn with_current_user<F: FnOnce(&User) -> T, T>(f: F) -> Option<T> {
    with_user_by_id(&current_uid(), f).await
}

/// Returns UID of current user.
pub fn current_uid() -> Uid {
    Uid::try_from(unsafe { libc::getuid() as usize }).unwrap()
}

struct OptionalI64Visitor;
impl<'de> serde::de::Visitor<'de> for OptionalI64Visitor {
    type Value = Option<i64>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an integer between -2^63 and 2^63")
    }

    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Some(value as _))
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Some(value as _))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Some(value))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: Error, {
        Ok(None)
    }
}

pub fn serialize_uid<S: serde::Serializer>(uid: &Uid, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_i64(**uid as _)
}

pub fn serialize_option_uid<S: serde::Serializer>(uid: &Option<Uid>, serializer: S) -> Result<S::Ok, S::Error> {
    match uid {
        Some(x) => serialize_uid(x, serializer),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize_uid<'a, D: serde::Deserializer<'a>>(deserializer: D) -> Result<Uid, D::Error> {
    match deserialize_option_uid(deserializer) {
        Ok(Some(x)) => Ok(x),
        Ok(None) => Err(D::Error::custom("empty")),
        Err(x) => Err(x),
    }
}

pub fn deserialize_option_uid<'a, D: serde::Deserializer<'a>>(deserializer: D) -> Result<Option<Uid>, D::Error> {
    let number = match deserializer.deserialize_i64(OptionalI64Visitor)? {
        Some(x) => x,
        None => return Ok(None),
    };
    Ok(
        Some(Uid::try_from(number as usize)
            .map_err(|_| D::Error::custom("invalid uid"))?),
    )
}

pub fn serialize_gid<S: serde::Serializer>(uid: &Gid, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_i64(**uid as _)
}

pub fn serialize_option_gid<S: serde::Serializer>(gid: &Option<Gid>, serializer: S) -> Result<S::Ok, S::Error> {
    match gid {
        Some(x) => serialize_gid(x, serializer),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize_gid<'a, D: serde::Deserializer<'a>>(deserializer: D) -> Result<Gid, D::Error> {
    match deserialize_option_gid(deserializer) {
        Ok(Some(x)) => Ok(x),
        Ok(None) => Err(D::Error::custom("empty")),
        Err(x) => Err(x),
    }
}

pub fn deserialize_option_gid<'a, D: serde::Deserializer<'a>>(deserializer: D) -> Result<Option<Gid>, D::Error> {
    let number = match deserializer.deserialize_i64(OptionalI64Visitor)? {
        Some(x) => x,
        None => return Ok(None),
    };
    Ok(
        Some(Gid::try_from(number as usize)
            .map_err(|_| D::Error::custom("invalid uid"))?),
    )
}
