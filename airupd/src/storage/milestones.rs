use airupfx::{
    files::{Milestone, ReadError},
    prelude::*,
};

/// Represents to Airup's milestones directory.
#[derive(Debug)]
pub struct Milestones {
    base_chain: DirChain,
}
impl From<DirChain> for Milestones {
    fn from(val: DirChain) -> Self {
        Self { base_chain: val }
    }
}
impl Milestones {
    pub fn new() -> Self {
        Self {
            base_chain: DirChain::from(airupfx::config::build_manifest().milestone_dir.clone()),
        }
    }

    /// Attempts to find and parse a milestone.
    pub async fn get(&self, name: &str) -> Result<Milestone, ReadError> {
        let name = name.strip_suffix(Milestone::SUFFIX).unwrap_or(name);
        match self
            .base_chain
            .find(format!("{name}{}", Milestone::SUFFIX))
            .await
        {
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