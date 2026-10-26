//! The MCP tools' backend over the server's services: the docs of the store, the live config,
//! the sync service's post path (source `mcp`), the run service, the notifications and the
//! health answer.

mod config;
mod docs;
mod errors;
mod ping;
mod posts;
mod runs;

use std::path::PathBuf;
use std::sync::Arc;

use argus_core::config::{AttachConfig, McpConfig};
use argus_mcp::{Image, LogQuery, McpBackend, NotifyRequest, RunFilter, StartOpts, ToolError};
use async_trait::async_trait;
use serde_json::Value;

use crate::Services;
use crate::routes::admin::health_body;

/// The tools' backend: every call reads or acts through these services.
pub struct ServerBackend {
    svc: Arc<Services>,
}

impl ServerBackend {
    /// A backend over the services.
    #[must_use]
    pub fn new(svc: Arc<Services>) -> Self {
        Self { svc }
    }
}

#[async_trait]
impl McpBackend for ServerBackend {
    fn project_keys(&self) -> Vec<String> {
        docs::project_keys(&self.svc)
    }

    fn docs(&self, scope: &str, coll: &str) -> Vec<Value> {
        docs::docs(&self.svc, scope, coll)
    }

    fn doc(&self, scope: &str, coll: &str, id: &str) -> Option<Value> {
        docs::doc(&self.svc, scope, coll, id)
    }

    fn policy(&self) -> McpConfig {
        config::policy(&self.svc)
    }

    fn attach_config(&self) -> AttachConfig {
        config::attach(&self.svc)
    }

    fn file_roots(&self) -> Vec<PathBuf> {
        config::file_roots(&self.svc)
    }

    fn default_base_url(&self) -> String {
        config::base_url(&self.svc)
    }

    fn cards(&self) -> Vec<Value> {
        docs::cards(&self.svc)
    }

    fn run_defs(&self, key: &str) -> Option<Vec<Value>> {
        docs::run_defs(&self.svc, key)
    }

    fn runs_up(&self) -> bool {
        runs::up(&self.svc)
    }

    fn run_get(&self, key: &str, id: &str) -> Option<Value> {
        runs::get(&self.svc, key, id)
    }

    fn run_list(&self, key: Option<&str>, filter: &RunFilter) -> Vec<Value> {
        runs::list(&self.svc, key, filter)
    }

    fn run_logs(&self, key: &str, id: &str, query: &LogQuery) -> Value {
        runs::logs(&self.svc, key, id, query)
    }

    async fn run_start(
        &self,
        key: &str,
        name: &str,
        opts: &StartOpts,
    ) -> Result<(Value, bool), ToolError> {
        runs::start(&self.svc, key, name, opts)
    }

    async fn run_stop(&self, key: &str, id: &str, by: &str) -> Result<Value, ToolError> {
        runs::stop(&self.svc, key, id, by).await
    }

    async fn run_attach(
        &self,
        key: &str,
        id: &str,
        image: &Image,
        caption: &str,
        by: &str,
    ) -> Result<Value, ToolError> {
        runs::attach(&self.svc, (key, id), image, caption, by)
    }

    async fn post_entry(&self, key: &str, entry: Value, via: &str) -> Result<Value, ToolError> {
        posts::entry(&self.svc, key, entry, via).await
    }

    async fn send_to_lead(
        &self,
        target: &str,
        text: &str,
        decision: bool,
        via: &str,
    ) -> Result<Value, ToolError> {
        posts::lead(&self.svc, target, text, decision, via).await
    }

    async fn attach_proof(
        &self,
        key: &str,
        test: &str,
        image: &Image,
        caption: &str,
        via: &str,
    ) -> Result<Value, ToolError> {
        posts::proof(&self.svc, (key, test), image, caption, via).await
    }

    async fn notify(&self, request: &NotifyRequest) -> Result<Value, ToolError> {
        ping::notify(&self.svc, request).await
    }

    async fn health(&self) -> Value {
        health_body(&self.svc).unwrap_or_else(|e| e.body())
    }
}
