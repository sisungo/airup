use std::sync::RwLock;
use once_cell::sync::Lazy;
use serde::de::Error;
use sysinfo::{Gid, SystemExt, Uid, User, UserExt};

static USERS: Lazy<RwLock<sysinfo::System>> = Lazy::new(|| {
    let mut instance = sysinfo::System::new();
    instance.refresh_users_list();
    instance.into()
});
static CACHE: Lazy<mini_moka::sync::Cache<Request, Option<usize>, ahash::RandomState>> =
    Lazy::new(|| {
        mini_moka::sync::Cache::builder()
            .initial_capacity(4)
            .max_capacity(64)
            .build_with_hasher(Default::default())
    });

/// Refreshes users database.
pub fn refresh() {
    USERS.write().unwrap().refresh_users_list();
    CACHE.invalidate_all();
}

/// Finds a user entry by UID.
pub fn with_user_by_id<F: FnOnce(&User) -> T, T>(uid: &Uid, f: F) -> Option<T> {
    let users = USERS.read().unwrap();
    let req = Request::FindUserById(uid.clone());
    let numeric = CACHE.get(&req).unwrap_or_else(|| {
        let value = users
            .users()
            .iter()
            .enumerate()
            .find(|(_, u)| u.id() == uid)
            .map(|x| x.0);
        CACHE.insert(req, value);
        value
    });
    numeric.map(|i| f(users.users().get(i).unwrap()))
}

/// Finds a user entry by username.
pub fn with_user_by_name<F: FnOnce(&User) -> T, T>(name: &str, f: F) -> Option<T> {
    let users = USERS.read().unwrap();
    let req = Request::FindUserByName(name.into());
    let numeric = CACHE.get(&req).unwrap_or_else(|| {
        let value = users
            .users()
            .iter()
            .enumerate()
            .find(|(_, u)| u.name() == name)
            .map(|x| x.0);
        CACHE.insert(req, value);
        value
    });
    numeric.map(|i| f(users.users().get(i).unwrap()))
}

/// Returns the user entry of current user.
pub fn with_current_user<F: FnOnce(&User) -> T, T>(f: F) -> Option<T> {
    with_user_by_id(&current_uid(), f)
}

/// Returns UID of current user.
pub fn current_uid() -> Uid {
    Uid::try_from(unsafe { libc::getuid() as usize }).unwrap()
}

struct I64Visitor;
impl<'de> serde::de::Visitor<'de> for I64Visitor {
    type Value = i64;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an integer between -2^63 and 2^63")
    }

    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(value as _)
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(value as _)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(value)
    }
}

pub fn serialize_uid<S: serde::Serializer>(uid: &Uid, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_i64(**uid as _)
}

pub fn serialize_option_uid<S: serde::Serializer>(
    uid: &Option<Uid>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
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

pub fn deserialize_option_uid<'a, D: serde::Deserializer<'a>>(
    deserializer: D,
) -> Result<Option<Uid>, D::Error> {
    let number = deserializer.deserialize_i64(I64Visitor)?;
    Ok(Some(
        Uid::try_from(number as usize).map_err(|_| D::Error::custom("invalid uid"))?,
    ))
}

pub fn serialize_gid<S: serde::Serializer>(uid: &Gid, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_i64(**uid as _)
}

pub fn serialize_option_gid<S: serde::Serializer>(
    gid: &Option<Gid>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
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

pub fn deserialize_option_gid<'a, D: serde::Deserializer<'a>>(
    deserializer: D,
) -> Result<Option<Gid>, D::Error> {
    let number = deserializer.deserialize_i64(I64Visitor)?;
    Ok(Some(
        Gid::try_from(number as usize).map_err(|_| D::Error::custom("invalid uid"))?,
    ))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Request {
    FindUserById(Uid),
    FindUserByName(String),
}
