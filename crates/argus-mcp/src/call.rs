//! One tool call as the handlers see it.

use serde_json::Value;

use crate::backend::McpBackend;
use crate::context::CallContext;

/// A validated call: the backend, the project it resolved to, the arguments and the names
/// writes are stamped with.
pub struct Call<'a> {
    /// The services and data.
    pub backend: &'a dyn McpBackend,
    /// The project key, none for the tools that take none.
    pub key: Option<String>,
    /// The validated arguments.
    pub args: &'a Value,
    /// `mcp:<client>`.
    pub via: String,
    /// The base of proof urls.
    pub base: String,
    /// The transport context.
    pub ctx: &'a CallContext,
}

impl Call<'_> {
    /// The resolved project key (empty for the tools that have none).
    #[must_use]
    pub fn key(&self) -> &str {
        self.key.as_deref().unwrap_or_default()
    }
}
