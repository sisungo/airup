//! Management of the Airup secure policy database.

#![allow(unstable_name_collisions)]

mod raw;

use ahash::AHashMap;
pub use raw::{Action, Actions};

use self::raw::{Policy, Subject, Verb};
use crate::prelude::*;
use std::sync::RwLock;

/// Represents to a policy database on the filesystem.
#[derive(Debug)]
pub struct Db {
    base_chain: DirChain,
    compiled: RwLock<Compiled>,
}
impl Db {
    /// Creates a new `Db` from provided chain.
    pub async fn new<C: Into<DirChain>>(chain: C) -> Self {
        let base_chain = chain.into();
        let policy = Self::read_policy(&base_chain).await;
        let compiled = RwLock::new(policy.into());
        Self {
            base_chain,
            compiled,
        }
    }

    /// Refreshes the cache.
    pub async fn refresh(&self) {
        *self.compiled.write().unwrap() = Self::read_policy(&self.base_chain).await.into();
    }

    /// Reads a policy from a directory chain.
    pub async fn read_policy(base_chain: &DirChain) -> Policy {
        let mut policy = Policy::with_preset();

        if let Ok(read_chain) = base_chain
            .read_chain()
            .await
            .inspect_err(|x| tracing::warn!("failed to get policy list: {x}"))
        {
            for i in read_chain {
                if i.to_string_lossy().ends_with(Policy::SUFFIX) {
                    if let Some(path) = base_chain
                        .find(&i)
                        .await
                        .inspect_none(|| tracing::warn!("failed to get policy file at `{:?}`", i))
                    {
                        tokio::fs::read_to_string(&path)
                            .await
                            .inspect_err(|e| {
                                tracing::warn!("failed to read policy file at `{:?}`: {}", i, e)
                            })
                            .map(|x| {
                                x.parse()
                                    .inspect_err(|e| {
                                        tracing::warn!(
                                            "failed to parse policy file at `{:?}`: {}",
                                            i,
                                            e
                                        )
                                    })
                                    .map(|mut y| policy.merge(&mut y))
                            })
                            .ok();
                    }
                }
            }
        }

        policy
    }

    /// Returns `true` if provided user is permitted to perform the operation.
    pub fn check(&self, user: Uid, actions: &Actions) -> bool {
        self.compiled.read().unwrap().check(user, actions)
    }
}

/// Represents to a compiled policy database.
#[derive(Debug, Clone, Default)]
struct Compiled {
    user_allow: AHashMap<Uid, Actions>,
    group_allow: AHashMap<String, Actions>,
}
impl Compiled {
    /// Returns `true` if provided user is permitted to perform the operation.
    fn check(&self, user: Uid, actions: &Actions) -> bool {
        self.user_allow
            .get(&user)
            .map(|x| actions.is_subset(x))
            .unwrap_or_default()
            || find_user_by_uid(user)
                .inspect_none(|| tracing::warn!("no such user `uid={}`", user))
                .map(|entry| {
                    entry.groups.iter().any(|x| {
                        self.group_allow
                            .get(x)
                            .map(|y| actions.is_subset(y))
                            .unwrap_or_default()
                    })
                })
                .unwrap_or_default()
    }
}
impl From<Policy> for Compiled {
    fn from(pol: Policy) -> Self {
        let mut result = Self::default();
        for mut i in pol.0 {
            match i.verb {
                Verb::Allow => match i.subject {
                    Subject::Uid(u) => {
                        let set = result.user_allow.would_get(&u);
                        i.actions.drain().for_each(|x| {
                            set.insert(x);
                        });
                    }
                    Subject::User(u) => {
                        if let Some(x) = find_user_by_name(&u)
                            .inspect_none(|| tracing::warn!("no such user `{}`", u))
                        {
                            let set = result.user_allow.would_get(&x.uid);
                            i.actions.drain().for_each(|x| {
                                set.insert(x);
                            });
                        }
                    }
                    Subject::Group(g) => {
                        let set = result.group_allow.would_get(&g);
                        i.actions.drain().for_each(|x| {
                            set.insert(x);
                        });
                    }
                },
                Verb::Deny => match i.subject {
                    Subject::Uid(u) => {
                        let set = result.user_allow.would_get(&u);
                        i.actions.drain().for_each(|x| {
                            set.remove(&x);
                        });
                    }
                    Subject::User(u) => {
                        if let Some(x) = find_user_by_name(&u)
                            .inspect_none(|| tracing::warn!("no such user `{}`", u))
                        {
                            let set = result.user_allow.would_get(&x.uid);
                            i.actions.drain().for_each(|x| {
                                set.remove(&x);
                            });
                        }
                    }
                    Subject::Group(g) => {
                        let set = result.group_allow.would_get(&g);
                        i.actions.drain().for_each(|x| {
                            set.remove(&x);
                        });
                    }
                },
            }
        }
        result
    }
}
