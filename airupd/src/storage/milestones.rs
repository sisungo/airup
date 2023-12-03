use airup_sdk::files::{Milestone, ReadError};
use airupfx::prelude::*;

/// Represents to Airup's milestones directory.
#[derive(Debug)]
pub struct Milestones {
    base_chain: DirChain<'static>,
}
impl From<DirChain<'static>> for Milestones {
    fn from(val: DirChain<'static>) -> Self {
        Self { base_chain: val }
    }
}
impl Milestones {
    pub fn new() -> Self {
        Self {
            base_chain: DirChain::new(airup_sdk::build::manifest().milestone_dir.clone()),
        }
    }

    /// Attempts to find and parse a milestone.
    pub async fn get(&self, name: &str) -> Result<Milestone, ReadError> {
        let name = name.strip_suffix(".airm").unwrap_or(name);
        match self.base_chain.find(format!("{name}.airm")).await {
            Some(x) => Milestone::read_from(x).await,
            None => Err(std::io::ErrorKind::NotFound.into()),
        }
    }
}
impl Default for Milestones {
    fn default() -> Self {
        Self::new()
    }
}
