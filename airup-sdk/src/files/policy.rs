use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Policy(Vec<PolicyItem>);
impl Display for Policy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).expect("policies should always be serializable")
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyItem {
    condition: Condition,
    operation: Operation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    All(Vec<Condition>),
    Any(Vec<Condition>),
    Not(Box<Condition>),

    MatchUser(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    Allowed,
    Denied,
}
