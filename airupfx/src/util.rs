//! Extension to the standard library.

use std::{
    collections::HashMap,
    future::Future,
    hash::{BuildHasher, Hash},
    pin::Pin, ffi::CString,
};
use ahash::AHashSet;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// An extension for standard [Result] type to support logging.
#[cfg(feature = "process")]
pub trait ResultExt<T> {
    /// Returns the contained `Ok` value, consuming the `self` value.
    fn unwrap_log(self, why: &str) -> T;
}
#[cfg(feature = "process")]
impl<T, E> ResultExt<T> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn unwrap_log(self, why: &str) -> T {
        match self {
            Ok(val) => val,
            Err(err) => {
                tracing::error!(target: "console", "{}: {}", why, err);
                crate::process::emergency();
            }
        }
    }
}

/// An extension for standard [Option] type to support logging.
#[cfg(feature = "process")]
pub trait OptionExt<T> {
    /// Returns the contained `Ok` value, consuming the `self` value.
    fn unwrap_log(self, why: &str) -> T;
}
#[cfg(feature = "process")]
impl<T> OptionExt<T> for Option<T> {
    fn unwrap_log(self, why: &str) -> T {
        match self {
            Some(val) => val,
            None => {
                tracing::error!(target: "console", "{why}");
                crate::process::emergency();
            }
        }
    }
}

/// An extension of [Iterator].
pub trait IterExt<T> {
    /// Removes *all* duplicated elements from the iterator.
    fn dedup_all(&mut self) -> Vec<T>;

    /// Removes *all* duplicated elements from the iterator, not reserving order.
    fn dedup_all_unstable(&mut self) -> Vec<T>;
}
impl<T, I> IterExt<T> for I
where
    I: Iterator<Item = T>,
    T: Hash + PartialEq + Eq,
{
    fn dedup_all(&mut self) -> Vec<T> {
        let mut result = Vec::new();
        self.for_each(|x| {
            if !result.contains(&x) {
                result.push(x);
            }
        });
        result
    }

    fn dedup_all_unstable(&mut self) -> Vec<T> {
        let mut set = AHashSet::new();
        self.for_each(|x| {
            set.insert(x);
        });
        set.into_iter().collect()
    }
}

/// An extension to [HashMap].
pub trait HashMapExt<K, V> {
    /// Returns mutable reference of provided key, inserts default value if not existing.
    fn would_get(&mut self, key: &K) -> &mut V;
}
impl<K, V, H> HashMapExt<K, V> for HashMap<K, V, H>
where
    K: Clone + PartialEq + Eq + Hash,
    V: Default,
    H: BuildHasher,
{
    fn would_get(&mut self, key: &K) -> &mut V {
        match self.contains_key(key) {
            true => self.get_mut(key).unwrap(),
            false => {
                self.insert(key.clone(), Default::default());
                self.get_mut(key).unwrap()
            }
        }
    }
}

pub fn cstring_lossy(s: &str) -> CString {
    let s = s.replace('\0', "\u{fffd}").into_bytes();
    debug_assert!(s.iter().find(|x| **x == 0).is_none());
    unsafe { CString::from_vec_unchecked(s) }
}