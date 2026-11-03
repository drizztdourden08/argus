//! The sync service: start (post recovery, a full export, the watcher), stop, and what the
//! routes ask of it.

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, PoisonError, RwLock};

use argus_board::Board;
use argus_board::names::display_name;
use argus_board::ops::is_pr_kind;
use argus_board::portfolio::project_keys;
use argus_board::store::board_path;
use argus_core::ids::PORTFOLIO;
use argus_core::log::Logger;
use argus_core::{ApiError, Config, Paths, ResetEvent};
use argus_store::Db;
use serde::Serialize;
use serde_json::{Value, json};

use crate::hooks::Hooks;
use crate::posts::{PostResult, Source};
use crate::proofs::{Proof, ProofSource};
use crate::prs::PrApplied;
use crate::state::State;
use crate::watcher::{self, Watcher, is_board_key};
use crate::{RunsView, ScopeQueue};

/// What the sync service is built from.
#[derive(Clone)]
pub struct SyncOptions {
    /// The data folders.
    pub paths: Paths,
    /// The database.
    pub db: Arc<Db>,
    /// The live config.
    pub config: Arc<RwLock<Config>>,
    /// The server log.
    pub log: Arc<Logger>,
    /// Run counts for the cards, when the runs service is there.
    pub runs: Option<Arc<dyn RunsView>>,
    /// The per-scope mutation queues.
    pub queue: ScopeQueue,
}

/// One row of `/api/projects`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRow {
    /// The key.
    pub key: String,
    /// The display name.
    pub name: String,
    /// The project root, when known.
    pub root: Option<String>,
    /// A board file exists.
    pub has_board: bool,
    /// `/p/<key>/`.
    pub url: String,
}

/// What a resync touched.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Resynced {
    /// The scopes exported.
    pub scopes: Vec<String>,
    /// How many docs they hold.
    pub docs: usize,
}

/// The watcher's health.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WatcherHealth {
    /// Running and the file watcher is up.
    pub ok: bool,
    /// The last board event.
    pub last_event_at: Option<String>,
    /// Unreadable board reads.
    pub errors: u64,
}

/// The sync service.
pub struct SyncService {
    state: Arc<State>,
    watcher: Mutex<Option<Watcher>>,
}

impl SyncService {
    /// A stopped service.
    #[must_use]
    pub fn new(opts: SyncOptions) -> Self {
        SyncService {
            state: Arc::new(State::new(opts)),
            watcher: Mutex::new(None),
        }
    }

    fn board_keys(&self) -> Vec<String> {
        let Ok(dirs) = std::fs::read_dir(self.state.coord_home()) else {
            return Vec::new();
        };
        let mut keys: Vec<String> = dirs
            .filter_map(Result::ok)
            .filter_map(|d| d.file_name().to_str().map(str::to_owned))
            .filter(|k| {
                is_board_key(k) && k != PORTFOLIO && board_path(self.state.coord_home(), k).exists()
            })
            .collect();
        keys.sort();
        keys
    }

    /// Post recovery, a full export of every board and the portfolio, then the watcher.
    pub async fn start(&self) {
        self.state.stopped.store(false, Ordering::SeqCst);
        self.state.recover_posts().await;
        for key in self.board_keys() {
            self.state.export_now(&key).await;
        }
        self.state.export_now(PORTFOLIO).await;
        *self.watcher.lock().unwrap_or_else(PoisonError::into_inner) =
            Some(watcher::start(&self.state));
    }

    /// Calls `listener` with the key each time the board of a project was exported, whether or
    /// not its docs changed. The listener runs on the exporting thread, so it returns quickly
    /// and hands slow work to another thread.
    pub fn on_board(&self, listener: impl Fn(&str) + Send + Sync + 'static) {
        self.state
            .listeners
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(Arc::new(listener));
    }

    /// Stops the watcher and every pending export.
    pub fn stop(&self) {
        self.state.stopped.store(true, Ordering::SeqCst);
        if let Some(w) = self
            .watcher
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
        {
            w.stop();
        }
    }

    fn registry(&self) -> Value {
        self.state
            .board(PORTFOLIO)
            .map(|p| p["projects"].clone())
            .filter(Value::is_object)
            .unwrap_or_else(|| json!({}))
    }

    /// Every project: the registry plus the boards found, retired ones left out.
    #[must_use]
    pub fn projects(&self) -> Vec<ProjectRow> {
        let reg = self.registry();
        let mut keys = project_keys(self.state.coord_home(), &json!({ "projects": reg }));
        keys.extend(self.board_keys());
        keys.sort();
        keys.dedup();
        let config = self.state.config();
        keys.into_iter()
            .filter(|k| reg[k]["retired"] != json!(true))
            .map(|key| {
                let board = self.state.board(&key);
                ProjectRow {
                    name: display_name(
                        &key,
                        board.as_ref().and_then(|b| b["name"].as_str()),
                        reg[&key]["name"].as_str(),
                    ),
                    root: config
                        .project_roots
                        .get(&key)
                        .cloned()
                        .or_else(|| reg[&key]["root"].as_str().map(str::to_owned)),
                    has_board: board.is_some()
                        || board_path(self.state.coord_home(), &key).exists(),
                    url: format!("/p/{key}/"),
                    key,
                }
            })
            .collect()
    }

    /// The hooks other services hang on this one.
    #[must_use]
    pub fn hooks(&self) -> &Hooks {
        &self.state.hooks
    }

    /// The last good board of a key.
    #[must_use]
    pub fn board(&self, key: &str) -> Option<Board> {
        self.state.board(key)
    }

    /// The run definitions of a project, each with its name.
    #[must_use]
    pub fn run_defs(&self, key: &str) -> Vec<Value> {
        let board = self.state.board(key);
        let defs = board.as_ref().and_then(|b| b["runDefs"].as_object());
        defs.into_iter()
            .flatten()
            .map(|(name, def)| {
                let mut d = def.clone();
                if let Some(o) = d.as_object_mut() {
                    o.insert("name".into(), json!(name));
                }
                d
            })
            .collect()
    }

    /// Applies an owner post.
    ///
    /// # Errors
    /// The Node errors: 400 bad post, 404 no board, 503 board locked.
    pub async fn post_entry(
        &self,
        scope: &str,
        body: &Value,
        source: Source,
        via: Option<&str>,
    ) -> Result<PostResult, ApiError> {
        self.state.post_entry(scope, body, source, via).await
    }

    /// The owner sends a note or a decision down to a lead.
    ///
    /// # Errors
    /// As [`Self::post_entry`].
    pub async fn send_to_lead(
        &self,
        project: &str,
        text: &str,
        decision: bool,
        via: &str,
    ) -> Result<PostResult, ApiError> {
        self.state.send_to_lead(project, text, decision, via).await
    }

    /// Keeps a proof image on a test.
    ///
    /// # Errors
    /// 400, 403, 404, 413 or 415 as the Node service answers.
    pub async fn attach_proof(
        &self,
        key: &str,
        test_id: &str,
        src: ProofSource,
        caption: &str,
        by: &str,
    ) -> Result<Proof, ApiError> {
        self.state
            .attach_proof(key, test_id, src, caption, by)
            .await
    }

    /// Keeps a picture on a question or a decision entry, tied to a 1-based option or to none.
    ///
    /// # Errors
    /// 400, 403, 404, 413 or 415 as [`Self::attach_proof`] answers.
    pub async fn attach_entry_image(
        &self,
        key: &str,
        target: (&str, Option<i64>),
        pic: (ProofSource, &str),
        by: &str,
    ) -> Result<Proof, ApiError> {
        self.state.attach_entry_image(key, target, pic, by).await
    }

    /// Keeps a picture on an item and lists it in the item's `images`.
    ///
    /// # Errors
    /// 400, 403, 404, 413 or 415 as [`Self::attach_proof`] answers.
    pub async fn attach_item_image(
        &self,
        key: &str,
        item_id: &str,
        pic: (ProofSource, &str),
        by: &str,
    ) -> Result<Proof, ApiError> {
        self.state.attach_item_image(key, item_id, pic, by).await
    }

    /// Applies one pull request op (`pr-open`, `pr-update`, `pr-merge`, `pr-close`,
    /// `pr-checks`) to the `prs` collection as `by`, and exports the board.
    ///
    /// # Errors
    /// 400 for a bad op or one the board refuses, 404 for a project with no board, 503 for a
    /// board that stays locked.
    pub async fn apply_pr(&self, key: &str, op: Value, by: &str) -> Result<PrApplied, ApiError> {
        if !op["kind"].as_str().is_some_and(is_pr_kind) {
            return Err(ApiError::bad_request("not a pull request op"));
        }
        self.state.apply_pr(key, op, by).await
    }

    /// Exports again (soft: hash diff; hard: clear, export, reset the streams).
    ///
    /// # Errors
    /// 400 for a bad project, 404 for a project with no board.
    pub async fn resync(&self, project: Option<&str>, hard: bool) -> Result<Resynced, ApiError> {
        if let Some(p) = project.filter(|p| !is_board_key(p)) {
            return Err(ApiError::bad_request(format!("invalid project \"{p}\"")));
        }
        let scopes = project.map_or_else(
            || {
                self.board_keys()
                    .into_iter()
                    .chain([PORTFOLIO.to_owned()])
                    .collect()
            },
            |p| vec![p.to_owned()],
        );
        let mut docs = 0;
        let mut done = Vec::new();
        for scope in scopes {
            if scope != PORTFOLIO && !board_path(self.state.coord_home(), &scope).exists() {
                if project.is_some() {
                    return Err(ApiError::not_found(format!(
                        "no board for project \"{scope}\""
                    )));
                }
                continue;
            }
            docs += self.resync_scope(&scope, hard).await?;
            done.push(scope);
        }
        Ok(Resynced { scopes: done, docs })
    }

    async fn resync_scope(&self, scope: &str, hard: bool) -> Result<usize, ApiError> {
        let state = Arc::clone(&self.state);
        self.state
            .opts
            .queue
            .run(scope, async move {
                if hard {
                    state.opts.db.docs().clear(scope)?;
                }
                let out = if scope == PORTFOLIO {
                    state.refresh_portfolio().await?
                } else {
                    state.refresh(scope).await?
                };
                if hard {
                    state.opts.db.bus().emit_reset(&ResetEvent {
                        scope: scope.to_owned(),
                        reason: "resync".into(),
                    });
                }
                Ok(out.map_or(0, |o| o.docs))
            })
            .await
    }

    /// The current portfolio cards.
    #[must_use]
    pub fn cards(&self) -> Vec<Value> {
        self.state
            .cards
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// The watcher's health.
    #[must_use]
    pub fn health(&self) -> WatcherHealth {
        let fs_ok = self
            .watcher
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
            .is_some_and(Watcher::ok);
        WatcherHealth {
            ok: !self.state.stopped() && fs_ok,
            last_event_at: self
                .state
                .last_event_at
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .clone(),
            errors: self.state.errors.load(Ordering::SeqCst),
        }
    }
}
