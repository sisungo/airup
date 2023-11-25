//! Logger interface of the Airup supervisor.

/// Represents to a client of a logger system.
#[async_trait::async_trait]
pub trait Logger {}
