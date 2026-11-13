//! An in-memory [`McpBackend`] for tests: docs from a JSON tree (`scope -> collection -> id ->
//! doc`), a fixed set of runs, and every write recorded in [`Calls`] instead of done.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

use argus_core::config::{AttachConfig, McpConfig};
use async_trait::async_trait;
use serde_json::{Map, Value, json};

use crate::backend::{Image, LogQuery, McpBackend, NotifyRequest, RunFilter, StartOpts};
use crate::error::ToolError;
use crate::json::text;
use crate::memory_runs::{filter_runs, log_chunk};

/// The writes the backend was asked to do, as JSON, oldest first.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Calls {
    /// `post_entry`: `{scope, body, source, via}`.
    pub post: Vec<Value>,
    /// `send_to_lead`: `{project, text, decision, via}`.
    pub send: Vec<Value>,
    /// `attach_proof`: `{key, testId, bytes, mime, caption, by}`.
    pub proof: Vec<Value>,
    /// `run_start`: `{key, name, origin, by, visible, agent, reason}`.
    pub run_start: Vec<Value>,
    /// `run_stop`: `{key, rid, by}`.
    pub run_stop: Vec<Value>,
    /// `run_attach`: `{key, rid, bytes, mime, caption, by}`.
    pub run_attach: Vec<Value>,
    /// `notify`: `{title, text, level, project, thread}`.
    pub notify: Vec<Value>,
    /// `dep_cancel`: `{id, reason}`.
    pub dep_cancel: Vec<Value>,
}

struct RunsState {
    rows: Vec<Value>,
    lines: Vec<Value>,
}

/// The in-memory backend.
pub struct MemoryBackend {
    docs: Value,
    runs: Option<RunsState>,
    run_defs: Option<Vec<Value>>,
    sync_up: bool,
    notify_up: bool,
    deps_up: bool,
    policy: Mutex<McpConfig>,
    attach: AttachConfig,
    roots: Vec<PathBuf>,
    home: Option<PathBuf>,
    calls: Mutex<Calls>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl MemoryBackend {
    /// A backend over `docs`, with no runs service, the sync service up, notifications off,
    /// every tool allowed and a 1 MiB attach cap.
    #[must_use]
    pub fn new(docs: Value) -> Self {
        Self {
            docs,
            runs: None,
            run_defs: None,
            sync_up: true,
            notify_up: false,
            deps_up: true,
            policy: Mutex::new(McpConfig {
                allow: json!("*"),
                deny: Vec::new(),
                read_only: false,
                extra: Map::new(),
            }),
            attach: AttachConfig {
                max_bytes: 1024 * 1024,
                extra_roots: Vec::new(),
                extra: Map::new(),
            },
            roots: Vec::new(),
            home: None,
            calls: Mutex::new(Calls::default()),
        }
    }

    /// Turn the runs service on with these run rows (each carries `scope`) and the one log
    /// every run shares.
    #[must_use]
    pub fn with_runs(mut self, rows: Vec<Value>, lines: Vec<Value>) -> Self {
        self.runs = Some(RunsState { rows, lines });
        self
    }

    /// Serve these run definitions for every project instead of the `launches` docs.
    #[must_use]
    pub fn with_run_defs(mut self, defs: Vec<Value>) -> Self {
        self.run_defs = Some(defs);
        self
    }

    /// Turn the sync service off: posts, notes and proofs answer `unavailable`.
    #[must_use]
    pub fn without_sync(mut self) -> Self {
        self.sync_up = false;
        self
    }

    /// Turn notifications on.
    #[must_use]
    pub fn with_notify(mut self) -> Self {
        self.notify_up = true;
        self
    }

    /// Turn dependencies off: the two dependency tools are hidden and refused.
    #[must_use]
    pub fn without_deps(mut self) -> Self {
        self.deps_up = false;
        self
    }

    /// Set the attach limits.
    #[must_use]
    pub fn with_attach(mut self, attach: AttachConfig) -> Self {
        self.attach = attach;
        self
    }

    /// Set the coord home that holds `boards.json`.
    #[must_use]
    pub fn with_home(mut self, home: PathBuf) -> Self {
        self.home = Some(home);
        self
    }

    /// Set the folders a proof path may live in.
    #[must_use]
    pub fn with_roots(mut self, roots: Vec<PathBuf>) -> Self {
        self.roots = roots;
        self
    }

    /// Replace `config.mcp`; the next call sees it.
    pub fn set_policy(&self, policy: McpConfig) {
        *lock(&self.policy) = policy;
    }

    /// The writes recorded so far.
    #[must_use]
    pub fn calls(&self) -> Calls {
        lock(&self.calls).clone()
    }

    fn record(&self, write: impl FnOnce(&mut Calls)) {
        write(&mut lock(&self.calls));
    }

    fn need_sync(&self) -> Result<(), ToolError> {
        if self.sync_up {
            Ok(())
        } else {
            Err(ToolError::unavailable("the sync service is not running"))
        }
    }

    fn rows(&self) -> Result<&RunsState, ToolError> {
        self.runs
            .as_ref()
            .ok_or_else(|| ToolError::unavailable("the runs service is not running"))
    }
}

#[async_trait]
impl McpBackend for MemoryBackend {
    fn project_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self
            .docs
            .as_object()
            .into_iter()
            .flatten()
            .filter(|(scope, colls)| scope.as_str() != "_portfolio" && colls["meta"].is_object())
            .map(|(scope, _)| scope.clone())
            .collect();
        keys.sort();
        keys
    }

    fn docs(&self, scope: &str, coll: &str) -> Vec<Value> {
        self.docs[scope][coll]
            .as_object()
            .into_iter()
            .flat_map(|m| m.values().cloned())
            .collect()
    }

    fn doc(&self, scope: &str, coll: &str, id: &str) -> Option<Value> {
        self.docs[scope][coll].get(id).cloned()
    }

    fn home(&self) -> Option<PathBuf> {
        self.home.clone()
    }

    fn policy(&self) -> McpConfig {
        lock(&self.policy).clone()
    }

    fn attach_config(&self) -> AttachConfig {
        self.attach.clone()
    }

    fn file_roots(&self) -> Vec<PathBuf> {
        self.roots.clone()
    }

    fn default_base_url(&self) -> String {
        "http://127.0.0.1:47615".to_owned()
    }

    fn cards(&self) -> Vec<Value> {
        vec![json!({ "id": "alpha" })]
    }

    fn run_defs(&self, _key: &str) -> Option<Vec<Value>> {
        self.run_defs.clone()
    }

    fn runs_up(&self) -> bool {
        self.runs.is_some()
    }

    fn run_get(&self, key: &str, id: &str) -> Option<Value> {
        let runs = self.runs.as_ref()?;
        let found = |r: &&Value| text(r, "scope") == Some(key) && text(r, "id") == Some(id);
        runs.rows.iter().find(found).cloned()
    }

    fn run_list(&self, key: Option<&str>, filter: &RunFilter) -> Vec<Value> {
        self.runs
            .as_ref()
            .map_or_else(Vec::new, |r| filter_runs(&r.rows, key, filter))
    }

    fn run_logs(&self, _key: &str, _id: &str, query: &LogQuery) -> Value {
        self.runs
            .as_ref()
            .map_or(Value::Null, |r| log_chunk(&r.lines, query))
    }

    async fn run_start(
        &self,
        key: &str,
        name: &str,
        opts: &StartOpts,
    ) -> Result<(Value, bool), ToolError> {
        self.rows()?;
        let agent = opts.agent.as_ref().map_or(Value::Null, |a| json!(a));
        let reason = opts.reason.as_ref().map_or(Value::Null, |a| json!(a));
        self.record(|c| {
            c.run_start.push(json!({
                "key": key, "name": name, "origin": "mcp", "by": opts.by,
                "visible": false, "agent": agent, "reason": reason
            }));
        });
        if self.doc(key, "launches", name).is_none() {
            return Err(ToolError::new(
                "unknown_run",
                format!("no run definition \"{name}\" in {key}"),
            ));
        }
        let run = json!({ "id": "R-3", "name": name, "status": "queued", "visible": false,
            "origin": "mcp", "by": opts.by });
        Ok((run, false))
    }

    async fn run_stop(&self, key: &str, id: &str, by: &str) -> Result<Value, ToolError> {
        let rows = self.rows()?;
        self.record(|c| c.run_stop.push(json!({ "key": key, "rid": id, "by": by })));
        let mut run = rows
            .rows
            .iter()
            .find(|r| text(r, "id") == Some(id))
            .cloned()
            .unwrap_or_else(|| json!({}));
        if let Some(map) = run.as_object_mut() {
            map.insert("status".into(), json!("stopped"));
        }
        Ok(run)
    }

    async fn run_attach(
        &self,
        key: &str,
        id: &str,
        image: &Image,
        caption: &str,
        by: &str,
    ) -> Result<Value, ToolError> {
        self.rows()?;
        self.record(|c| {
            c.run_attach
                .push(json!({ "key": key, "rid": id, "bytes": image.bytes.len(),
                "mime": image.mime, "caption": caption, "by": by }));
        });
        Ok(json!({ "id": "att1", "url": format!("/files/{key}/a/att1") }))
    }

    async fn post_entry(&self, key: &str, entry: Value, via: &str) -> Result<Value, ToolError> {
        self.need_sync()?;
        let refused =
            text(&entry, "kind") == Some("ack") && text(&entry, "replyTo") == Some("NOPE");
        let mut stored = entry.clone();
        stored["status"] = json!("applied");
        let id = entry["id"].clone();
        self.record(|c| {
            c.post
                .push(json!({ "scope": key, "body": entry, "source": "mcp", "via": via }));
        });
        let status = if refused { "refused" } else { "applied" };
        Ok(json!({ "id": id, "status": status, "outcome": null, "entry": stored, "seq": 9 }))
    }

    async fn send_to_lead(
        &self,
        target: &str,
        text: &str,
        decision: bool,
        via: &str,
    ) -> Result<Value, ToolError> {
        self.need_sync()?;
        self.record(|c| {
            c.send
                .push(json!({ "project": target, "text": text, "decision": decision, "via": via }));
        });
        Ok(json!({ "id": "P-1", "status": "delivered", "outcome": null, "entry": null, "seq": 3 }))
    }

    async fn attach_proof(
        &self,
        key: &str,
        test: &str,
        image: &Image,
        caption: &str,
        via: &str,
    ) -> Result<Value, ToolError> {
        self.need_sync()?;
        self.record(|c| {
            c.proof.push(
                json!({ "key": key, "testId": test, "bytes": image.bytes.len(),
                "mime": image.mime, "caption": caption, "by": via }),
            );
        });
        Ok(json!({ "n": 2, "url": format!("/files/{key}/proofs/{test}-2.png") }))
    }

    async fn notify(&self, request: &NotifyRequest) -> Result<Value, ToolError> {
        if !self.notify_up {
            return Err(ToolError::unavailable("notifications are off"));
        }
        self.record(|c| {
            c.notify
                .push(json!({ "title": request.title, "text": request.text,
                "level": request.level, "project": request.project, "thread": request.thread }));
        });
        let toast = request.level != "info";
        Ok(json!({ "recorded": true, "id": "N-1", "level": request.level, "toast": toast }))
    }

    fn deps_enabled(&self) -> bool {
        self.deps_up
    }

    async fn dep_cancel(&self, id: &str, reason: Option<&str>) -> Result<Value, ToolError> {
        self.record(|c| c.dep_cancel.push(json!({ "id": id, "reason": reason })));
        Ok(json!({ "dep": { "id": id, "state": "cancelled" }, "text": format!("{id} cancelled") }))
    }

    async fn health(&self) -> Value {
        let counts = |status: &str| {
            self.run_list(
                None,
                &RunFilter {
                    status: Some(status.into()),
                    ..RunFilter::default()
                },
            )
            .len()
        };
        json!({ "ok": true, "version": env!("CARGO_PKG_VERSION"), "scopes": self.project_keys(),
            "watcher": null, "runs": { "running": counts("running"), "queued": counts("queued") } })
    }
}
