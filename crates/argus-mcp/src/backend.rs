//! What the tools read and do. The server implements [`McpBackend`] over its services; the
//! tests use [`crate::MemoryBackend`].
//!
//! Reads go through four primitives over the mirrored docs (`project_keys`, `docs`, `doc` and
//! the run reads): the tools filter, shape and sort them exactly as the Node tools did, so the
//! board on screen and the answer here match. Everything that changes data or speaks to the
//! owner is one method per service call and always acts as the owner, stamped with `via`.

use std::path::PathBuf;

use argus_core::config::{AttachConfig, McpConfig};
use async_trait::async_trait;
use serde_json::Value;

use crate::error::ToolError;

/// An image already checked against its magic bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    /// The file bytes.
    pub bytes: Vec<u8>,
    /// `image/png`, `image/jpeg`, `image/gif` or `image/webp`.
    pub mime: &'static str,
}

/// The filter of a run listing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunFilter {
    /// Only runs in this status.
    pub status: Option<String>,
    /// Only runs of this definition.
    pub name: Option<String>,
    /// Most rows.
    pub limit: Option<u64>,
}

/// Which lines of a run log to read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LogQuery {
    /// Lines after this sequence number.
    pub after: Option<u64>,
    /// The last N lines.
    pub tail: Option<u64>,
    /// `out`, `err` or `all`.
    pub stream: Option<String>,
}

/// How a run is started over MCP: origin `mcp`, never visible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartOpts {
    /// `mcp:<client>`.
    pub by: String,
    /// The sub-agent record the run belongs to.
    pub agent: Option<String>,
    /// Why the run was started.
    pub reason: Option<String>,
}

/// A ping to the owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifyRequest {
    /// The toast title.
    pub title: Option<String>,
    /// The text.
    pub text: String,
    /// `info`, `ping` or `urgent`.
    pub level: String,
    /// The project it is about.
    pub project: Option<String>,
    /// `mcp:<client>`, the caller that spoke.
    pub thread: String,
}

/// The services and data behind the tools.
#[async_trait]
pub trait McpBackend: Send + Sync {
    /// The keys of the projects that have a board, sorted.
    fn project_keys(&self) -> Vec<String>;

    /// Every doc of one collection of a scope, in store order.
    fn docs(&self, scope: &str, coll: &str) -> Vec<Value>;

    /// One doc.
    fn doc(&self, scope: &str, coll: &str, id: &str) -> Option<Value>;

    /// `config.mcp`, read on every call so an edit takes effect without a restart.
    fn policy(&self) -> McpConfig;

    /// `config.attach`: the size cap and the extra folders of a proof path.
    fn attach_config(&self) -> AttachConfig;

    /// The folders a proof path may live in besides the temp folder and the attach roots: the
    /// project roots and the coord home.
    fn file_roots(&self) -> Vec<PathBuf>;

    /// `http://127.0.0.1:<port>`, the base of proof urls when the request carried no Host.
    fn default_base_url(&self) -> String;

    /// The coord summary of the live board, when the sync service holds it.
    fn coord_summary(&self, _key: &str) -> Option<Value> {
        None
    }

    /// The portfolio cards when the docs hold none yet.
    fn cards(&self) -> Vec<Value> {
        Vec::new()
    }

    /// The run definitions of a project when the sync service has them, else the docs are used.
    fn run_defs(&self, _key: &str) -> Option<Vec<Value>> {
        None
    }

    /// Whether the runs service is running.
    fn runs_up(&self) -> bool;

    /// One run.
    fn run_get(&self, key: &str, id: &str) -> Option<Value>;

    /// Runs of a project, or of every project with no key; empty when the service is down.
    fn run_list(&self, key: Option<&str>, filter: &RunFilter) -> Vec<Value>;

    /// `{lines, next, dropped}` of a run.
    fn run_logs(&self, key: &str, id: &str, query: &LogQuery) -> Value;

    /// Start a defined run by name: the run and whether a live one answered instead.
    async fn run_start(
        &self,
        key: &str,
        name: &str,
        opts: &StartOpts,
    ) -> Result<(Value, bool), ToolError>;

    /// Stop a queued or running run.
    async fn run_stop(&self, key: &str, id: &str, by: &str) -> Result<Value, ToolError>;

    /// Attach a picture to a run: `{id, url}`.
    async fn run_attach(
        &self,
        key: &str,
        id: &str,
        image: &Image,
        caption: &str,
        by: &str,
    ) -> Result<Value, ToolError>;

    /// Post an entry as the owner (source `mcp`): the post result.
    async fn post_entry(&self, key: &str, entry: Value, via: &str) -> Result<Value, ToolError>;

    /// Send a note down to the lead of a project, or of all.
    async fn send_to_lead(
        &self,
        target: &str,
        text: &str,
        decision: bool,
        via: &str,
    ) -> Result<Value, ToolError>;

    /// Attach a picture to a test as proof: `{n, url}`.
    async fn attach_proof(
        &self,
        key: &str,
        test: &str,
        image: &Image,
        caption: &str,
        via: &str,
    ) -> Result<Value, ToolError>;

    /// Ping the owner: the notify result (`recorded`, `id`, `level`, `toast`, `why`).
    async fn notify(&self, request: &NotifyRequest) -> Result<Value, ToolError>;

    /// Whether cross-board dependencies are on (`deps.enabled`). The two dependency tools are
    /// hidden and refused while they are off.
    fn deps_enabled(&self) -> bool;

    /// Cancel a dependency as the owner: `{dep, text}`.
    async fn dep_cancel(&self, id: &str, reason: Option<&str>) -> Result<Value, ToolError>;

    /// The `health` tool result.
    async fn health(&self) -> Value;
}
