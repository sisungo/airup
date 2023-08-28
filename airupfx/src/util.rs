//! Extension to the standard library.

use std::{
    collections::HashMap,
    future::Future,
    hash::{BuildHasher, Hash},
    pin::Pin,
};

use ahash::AHashSet;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Port of unstable feature `#[feature(result_option_inspect)]` to stable Rust.
///
/// This will be deleted when the feature is stablized.
pub trait ResultExt<T, E> {
    fn inspect_err<F: FnOnce(&E)>(self, op: F) -> Self;
}
impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn inspect_err<F: FnOnce(&E)>(self, op: F) -> Self {
        self.map_err(|e| {
            op(&e);
            e
        })
    }
}

/// Port of unstable feature `#[feature(result_option_inspect)]` to stable Rust.
///
/// This will be deleted when the feature is stablized.
pub trait OptionExt<T> {
    fn inspect_none<F: FnOnce()>(self, op: F) -> Self;
}
impl<T> OptionExt<T> for Option<T> {
    fn inspect_none<F: FnOnce()>(self, op: F) -> Self {
        if self.is_none() {
            op()
        }

        self
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
