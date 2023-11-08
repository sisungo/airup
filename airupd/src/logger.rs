//! Logger interface of the Airup supervisor.
//!
//! This module implements an `airlog` client that can be integrated with the supervisor. It is disabled by default, and can
//! be enabled by RPC invocation.

/// Context of an `airlog` client.
#[derive(Debug)]
pub struct Logger {}
