//! Inspection of raw policy rules.

use crate::env::current_uid;
use anyhow::anyhow;
use std::{
    collections::HashSet,
    fmt::Display,
    ops::{Deref, DerefMut},
    str::FromStr,
};
use sysinfo::Uid;

/// Represents to a set of policy items.
#[derive(Debug, Clone, Default)]
pub struct Policy(pub Vec<Item>);
impl Policy {
    pub const EXTENSION: &'static str = "airp";
    pub const SUFFIX: &'static str = ".airp";

    /// Creates a new [Policy] with presets.
    pub fn with_preset() -> Self {
        format!("allow uid={} *; allow root *;", *current_uid())
            .parse()
            .unwrap()
    }

    /// Merges another policy to current policy, clearing the another one.
    pub fn merge(&mut self, another: &mut Policy) {
        self.0.append(&mut another.0);
    }
}
impl FromStr for Policy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result = Vec::new();
        for i in s.split(';') {
            let i = i.trim();
            if !i.is_empty() && !i.starts_with("//") {
                result.push(i.parse()?);
            }
        }
        Ok(Self(result))
    }
}

/// Represents to a policy item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub verb: Verb,
    pub subject: Subject,
    pub actions: Actions,
}
impl FromStr for Item {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splited = s.split_whitespace();
        let verb = splited
            .next()
            .ok_or_else(|| anyhow!("missing verb in policy item"))?
            .parse()?;
        let subject = splited
            .next()
            .ok_or_else(|| anyhow!("missing subject in policy item"))?
            .parse()?;
        let actions = splited
            .next()
            .ok_or_else(|| anyhow!("missing action in policy item"))?
            .parse()?;

        Ok(Self {
            verb,
            subject,
            actions,
        })
    }
}

/// Represents to a policy verb.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Verb {
    Allow,
    Deny,
}
impl FromStr for Verb {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "allow" => Ok(Self::Allow),
            "deny" => Ok(Self::Deny),
            _ => Err(anyhow!("policy verb `{s}` is unknown")),
        }
    }
}

/// Represents to a policy subject.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Subject {
    Uid(Uid),
    User(String),
    Group(String),
}
impl FromStr for Subject {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(u) = s.strip_prefix("user=") {
            Ok(Self::User(u.into()))
        } else if let Some(g) = s.strip_prefix("group=") {
            Ok(Self::Group(g.into()))
        } else if let Some(u) = s.strip_prefix("uid=") {
            Ok(Self::Uid(Uid::try_from(u.parse::<usize>()?)?))
        } else if s == "root" {
            Ok(Self::Uid(Uid::try_from(0).unwrap()))
        } else {
            Err(anyhow!("policy subject `{s}` is unknown"))
        }
    }
}

/// Represents to a policy action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Dump,
    Power,
    SideloadServices,
    Refresh,
    QueryServices,
    ManageServices,
}
impl Action {
    const STRING_MAP: &'static [(Self, &'static str)] = &[
        (Self::Dump, "dump"),
        (Self::Power, "power"),
        (Self::SideloadServices, "sideload_services"),
        (Self::Refresh, "refresh"),
        (Self::QueryServices, "query_services"),
        (Self::ManageServices, "manage_services"),
    ];
}
impl FromStr for Action {
    type Err = anyhow::Error;

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        Ok(Self::STRING_MAP
            .iter()
            .find(|(_, s)| *s == val)
            .ok_or_else(|| anyhow!("policy action `{val}` not found"))?
            .0)
    }
}
impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            Self::STRING_MAP
                .iter()
                .find(|(action, _)| self == action)
                .unwrap()
                .1
        )
    }
}

/// Represents to a set of policy actions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Actions(HashSet<Action>);
impl Actions {
    /// Returns an action set over all [Action]s.
    pub fn all() -> Self {
        Self(Action::STRING_MAP.iter().map(|x| x.0).collect())
    }
}
impl<T: Iterator<Item = Action>> From<T> for Actions {
    fn from(value: T) -> Self {
        let mut set = HashSet::new();
        value.for_each(|x| {
            set.insert(x);
        });
        Self(set)
    }
}
impl FromStr for Actions {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "*" {
            return Ok(Self::all());
        }

        let mut set = HashSet::new();
        for i in s.split(',') {
            set.insert(i.parse()?);
        }
        Ok(Self(set))
    }
}
impl Deref for Actions {
    type Target = HashSet<Action>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Actions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
