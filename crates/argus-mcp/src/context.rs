//! What the transport knows about one call: the default project, the client name and the base
//! url proof links are built on.

/// The context of one tool call or resource read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CallContext {
    /// The project of the client's configuration (`--project`), a valid key or none.
    pub default_project: Option<String>,
    /// The MCP client's name.
    pub client: String,
    /// `http://<Host>` of the request, when it had one.
    pub base_url: Option<String>,
}

impl CallContext {
    /// A context for a client with no default project and no base url.
    #[must_use]
    pub fn client(name: &str) -> Self {
        Self {
            default_project: None,
            client: name.to_owned(),
            base_url: None,
        }
    }

    /// `mcp:<client>`, the name board entries and runs are stamped with.
    #[must_use]
    pub fn via(&self) -> String {
        let client = if self.client.is_empty() {
            "client"
        } else {
            &self.client
        };
        let safe: String = client
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | ' ' | '-'))
            .take(40)
            .collect();
        let safe = safe.trim();
        format!("mcp:{}", if safe.is_empty() { "client" } else { safe })
    }
}
