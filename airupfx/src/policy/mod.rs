//! Management of the Airup secure policy database.

#![allow(unstable_name_collisions)]

mod raw;

pub use raw::{Action, Actions};

use self::raw::{Policy, Subject, Verb};
use crate::{env::with_user_by_name, prelude::*};
use ahash::AHashMap;
use sysinfo::{Uid, UserExt};

/// Represents to a policy database on the filesystem.
#[derive(Debug)]
pub struct Db {
    base_chain: DirChain<'static>,
    compiled: tokio::sync::RwLock<Compiled>,
}
impl Db {
    /// Creates a new `Db` from provided chain.
    pub async fn new<C: Into<DirChain<'static>>>(chain: C) -> Self {
        let base_chain = chain.into();
        let policy = Self::read_policy(&base_chain).await;
        let compiled = tokio::sync::RwLock::new(Compiled::from_policy(policy).await);
        Self {
            base_chain,
            compiled,
        }
    }

    /// Refreshes the cache.
    pub async fn refresh(&self) {
        *self.compiled.write().await =
            Compiled::from_policy(Self::read_policy(&self.base_chain).await).await;
    }

    /// Reads a policy from a directory chain.
    pub async fn read_policy(base_chain: &DirChain<'static>) -> Policy {
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
    pub async fn check(&self, user: &Uid, actions: &Actions) -> bool {
        self.compiled.read().await.check(user, actions).await
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
    async fn check(&self, user: &Uid, actions: &Actions) -> bool {
        self.user_allow
            .get(&user)
            .map(|x| actions.is_subset(x))
            .unwrap_or_default()
            || crate::env::with_user_by_id(&user, |entry| {
                entry.groups().iter().any(|x| {
                    self.group_allow
                        .get(x)
                        .map(|y| actions.is_subset(y))
                        .unwrap_or_default()
                })
            })
            .await
            .inspect_none(|| tracing::warn!("no such user `uid={}`", **user))
            .unwrap_or_default()
    }

    async fn from_policy(pol: Policy) -> Self {
        let mut result = Self::default();
        for mut i in pol.0 {
            match i.verb {
                Verb::Allow => match i.subject {
                    Subject::Uid(u) => {
                        let set = result.user_allow.get_or_default(&u);
                        i.actions.drain().for_each(|x| {
                            set.insert(x);
                        });
                    }
                    Subject::User(u) => {
                        crate::env::with_user_by_name(&u, |x| {
                            let set = result.user_allow.get_or_default(&x.id());
                            i.actions.drain().for_each(|x| {
                                set.insert(x);
                            });
                        })
                        .await
                        .inspect_none(|| tracing::warn!("no such user `{}`", u));
                    }
                    Subject::Group(g) => {
                        let set = result.group_allow.get_or_default(&g);
                        i.actions.drain().for_each(|x| {
                            set.insert(x);
                        });
                    }
                },
                Verb::Deny => match i.subject {
                    Subject::Uid(u) => {
                        let set = result.user_allow.get_or_default(&u);
                        i.actions.drain().for_each(|x| {
                            set.remove(&x);
                        });
                    }
                    Subject::User(u) => {
                        with_user_by_name(&u, |x| {
                            let set = result.user_allow.get_or_default(&x.id());
                            i.actions.drain().for_each(|x| {
                                set.remove(&x);
                            });
                        })
                        .await
                        .inspect_none(|| tracing::warn!("no such user `{}`", u));
                    }
                    Subject::Group(g) => {
                        let set = result.group_allow.get_or_default(&g);
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
