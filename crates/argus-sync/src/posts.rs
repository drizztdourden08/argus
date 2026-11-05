//! Owner posts to board ops. A post is checked, recorded in `posts` (its id makes it
//! idempotent), applied on the scope's queue under the board lock with the same import the CLI
//! replays, and exported before the answer: the answer's seq is that export's, so the page has
//! the change on its stream when the answer arrives. A post the server could not finish stays
//! `received` and is applied again at the next boot unless the board already has it.

use std::sync::Arc;

use argus_board::BoardError;
use argus_board::store::{LockWait, read_board, with_board_async};
use argus_board::sync::{import_entries_at, is_reply_kind};
use argus_core::ApiError;
use argus_core::clock::{now_iso, now_ms};
use argus_core::ids::{PORTFOLIO, is_scope};
use argus_store::StoreError;
use argus_store::rusqlite::{OptionalExtension, params};
use serde::Serialize;
use serde_json::{Value, json};

use crate::failure::board_error;
use crate::hooks::AppliedPost;
use crate::state::{State, clear_marks};
use crate::validate::{RESULT_NOTE_REQUIRED, result_note_missing, validate_entry};

/// Who posted: the owner's page, an MCP tool or the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// A browser page.
    Page,
    /// An MCP client.
    Mcp,
    /// The CLI or another non-browser caller.
    Cli,
}

impl Source {
    /// The wire name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Page => "page",
            Source::Mcp => "mcp",
            Source::Cli => "cli",
        }
    }
}

/// The answer to a post.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PostResult {
    /// The entry id.
    pub id: String,
    /// `applied`, `conflict`, `refused`, `duplicate`, `pending`, `delivered`...
    pub status: String,
    /// Why, when the board says.
    pub outcome: Option<String>,
    /// The entry as the board stored it.
    pub entry: Option<Value>,
    /// The seq of the export that carried it.
    pub seq: i64,
}

fn base36(mut n: u64) -> String {
    const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut out = Vec::new();
    loop {
        out.push(DIGITS[usize::try_from(n % 36).unwrap_or(0)]);
        n /= 36;
        if n == 0 {
            break;
        }
    }
    out.reverse();
    String::from_utf8(out).unwrap_or_default()
}

/// An owner post id, as the board page's `newId()` makes it: `O-<base36 ms>-<4 chars>`.
#[must_use]
pub fn new_post_id() -> String {
    let mut b = [0u8; 3];
    getrandom::fill(&mut b).unwrap_or_default();
    let tail = base36(u64::from(b[0]) << 16 | u64::from(b[1]) << 8 | u64::from(b[2]));
    let tail = format!("{tail:0>4}");
    format!(
        "O-{}-{}",
        base36(u64::try_from(now_ms()).unwrap_or(0)),
        &tail[tail.len() - 4..]
    )
}

struct PostRow {
    scope: String,
    body: String,
    status: String,
    outcome: Option<String>,
}

/// A reply that went through keeps the page's own status (`new`) on the board; the caller is
/// told `applied`.
fn result_of(id: &str, stored: Option<&Value>, seq: i64) -> PostResult {
    let Some(stored) = stored else {
        return PostResult {
            id: id.to_owned(),
            status: "refused".into(),
            outcome: Some("not applied (unknown kind or already on the board)".into()),
            entry: None,
            seq,
        };
    };
    let status = stored["status"].as_str().unwrap_or_default();
    let reply = is_reply_kind(stored["kind"].as_str().unwrap_or_default())
        && (status.is_empty() || status == "new");
    PostResult {
        id: id.to_owned(),
        status: if reply {
            "applied".into()
        } else {
            argus_core::js::string(&stored["status"])
        },
        outcome: stored["outcome"].as_str().map(str::to_owned),
        entry: Some(stored.clone()),
        seq,
    }
}

impl State {
    fn get_post(&self, id: &str) -> Option<PostRow> {
        self.opts.db.with_conn(|c| {
            c.query_row(
                "SELECT scope, body, status, outcome FROM posts WHERE id = ?",
                [id],
                |r| {
                    Ok(PostRow {
                        scope: r.get(0)?,
                        body: r.get(1)?,
                        status: r.get(2)?,
                        outcome: r.get(3)?,
                    })
                },
            )
            .optional()
            .ok()
            .flatten()
        })
    }

    fn insert_post(&self, e: &Value, scope: &str, source: Source) -> Result<(), StoreError> {
        let body = serde_json::to_string(e)?;
        self.opts.db.tx(|tx| {
            tx.conn().execute(
                "INSERT INTO posts(id, scope, kind, source, body, status, outcome, received_at, done_at) VALUES(?, ?, ?, ?, ?, 'received', NULL, ?, NULL)",
                params![e["id"].as_str(), scope, e["kind"].as_str(), source.as_str(), body, now_iso()],
            )?;
            Ok(())
        })
    }

    fn finish_post(&self, id: &str, status: &str, outcome: Option<&str>) {
        let done = self.opts.db.tx(|tx| {
            tx.conn().execute(
                "UPDATE posts SET status = ?, outcome = ?, done_at = ? WHERE id = ?",
                params![status, outcome, now_iso(), id],
            )?;
            Ok(())
        });
        if let Err(e) = done {
            self.warn("post status not recorded", id, &e);
        }
    }

    fn stored_result(&self, id: &str, row: &PostRow) -> Result<PostResult, ApiError> {
        let entry = self
            .board(&row.scope)
            .and_then(|b| b["entries"].get(id).cloned());
        Ok(PostResult {
            id: id.to_owned(),
            status: row.status.clone(),
            outcome: row.outcome.clone(),
            entry,
            seq: self.opts.db.docs().seq()?,
        })
    }

    async fn apply_project(
        self: &Arc<Self>,
        scope: &str,
        entry: Value,
    ) -> Result<PostResult, ApiError> {
        let me = Arc::clone(self);
        self.opts
            .queue
            .run(
                scope,
                async move { me.apply_project_now(scope, entry).await },
            )
            .await
    }

    async fn apply_project_now(
        self: &Arc<Self>,
        scope: &str,
        entry: Value,
    ) -> Result<PostResult, ApiError> {
        let id = entry["id"].as_str().unwrap_or_default().to_owned();
        let home = self.coord_home().to_path_buf();
        let before = self.board(scope);
        let known = |other: &str| other != id && self.get_post(other).is_some();
        let saved = with_board_async(self.coord_home(), scope, LockWait::default(), |board| {
            import_entries_at(&home, board, vec![entry], &known);
            clear_marks(board);
            Ok(board.clone())
        })
        .await;
        let saved = match saved {
            Ok(b) => b,
            Err(e) => return Err(self.post_failed(&id, e)),
        };
        let stored = saved["entries"].get(&id).cloned();
        let out = self.export_board(scope, saved.clone())?;
        let result = result_of(&id, stored.as_ref(), out.seq);
        self.finish_post(&id, &result.status, result.outcome.as_deref());
        self.hooks.applied(&AppliedPost {
            scope: scope.to_owned(),
            entry: stored,
            status: result.status.clone(),
            board: Some(saved),
            before,
        });
        Ok(result)
    }

    fn post_failed(&self, id: &str, e: BoardError) -> ApiError {
        if !matches!(e, BoardError::Locked(_)) {
            self.finish_post(id, "error", Some(&e.to_string()));
        }
        board_error(e)
    }

    async fn apply_portfolio(self: &Arc<Self>, entry: Value) -> Result<PostResult, ApiError> {
        let id = entry["id"].as_str().unwrap_or_default().to_owned();
        let (stored, status, seq) = match self.post_portfolio(entry).await {
            Ok(v) => v,
            Err(e) => {
                if e.code != argus_core::ErrorCode::BoardLocked {
                    self.finish_post(&id, "error", Some(&e.message));
                }
                return Err(e);
            }
        };
        let reason = stored
            .as_ref()
            .and_then(|e| e["reason"].as_str())
            .map(str::to_owned);
        self.finish_post(&id, &status, reason.as_deref());
        self.hooks.applied(&AppliedPost {
            scope: PORTFOLIO.to_owned(),
            entry: stored.clone(),
            status: status.clone(),
            board: None,
            before: None,
        });
        Ok(PostResult {
            id,
            status,
            outcome: reason,
            entry: stored,
            seq,
        })
    }

    async fn run_post(self: &Arc<Self>, scope: &str, entry: Value) -> Result<PostResult, ApiError> {
        if scope == PORTFOLIO {
            self.apply_portfolio(entry).await
        } else {
            self.apply_project(scope, entry).await
        }
    }

    /// Applies one owner post.
    pub async fn post_entry(
        self: &Arc<Self>,
        scope: &str,
        body: &Value,
        source: Source,
        via: Option<&str>,
    ) -> Result<PostResult, ApiError> {
        if !is_scope(scope) {
            return Err(ApiError::bad_request(format!("invalid scope \"{scope}\"")));
        }
        let entry = validate_entry(scope, body, source, via)?;
        let id = entry["id"].as_str().unwrap_or_default().to_owned();
        if let Some(row) = self.get_post(&id) {
            return self.known_post(scope, &id, row).await;
        }
        if scope != PORTFOLIO && !argus_board::store::board_path(self.coord_home(), scope).exists()
        {
            return Err(ApiError::not_found(format!(
                "no board for project \"{scope}\""
            )));
        }
        self.insert_post(&entry, scope, source)?;
        if scope != PORTFOLIO && entry["kind"] == "result" && result_note_missing(&entry) {
            self.finish_post(&id, "refused", Some(RESULT_NOTE_REQUIRED));
            return Ok(PostResult {
                id,
                status: "refused".into(),
                outcome: Some(RESULT_NOTE_REQUIRED.into()),
                entry: None,
                seq: self.opts.db.docs().seq()?,
            });
        }
        self.run_post(scope, entry).await
    }

    async fn known_post(
        self: &Arc<Self>,
        scope: &str,
        id: &str,
        row: PostRow,
    ) -> Result<PostResult, ApiError> {
        if row.scope != scope {
            return Err(ApiError::bad_request(format!(
                "entry id {id} was already used on {}",
                row.scope
            )));
        }
        if row.status != "received" {
            return self.stored_result(id, &row);
        }
        let entry: Value = serde_json::from_str(&row.body).map_err(|_| ApiError::internal())?;
        self.run_post(scope, entry).await
    }

    /// The owner sends a note or a decision down to a lead.
    pub async fn send_to_lead(
        self: &Arc<Self>,
        project: &str,
        text: &str,
        decision: bool,
        via: &str,
    ) -> Result<PostResult, ApiError> {
        let source = if via.starts_with("mcp") {
            Source::Mcp
        } else if via == "page" {
            Source::Page
        } else {
            Source::Cli
        };
        let kind = if decision { "decision" } else { "followup" };
        let body = json!({ "id": new_post_id(), "kind": kind, "project": project, "text": text });
        self.post_entry(PORTFOLIO, &body, source, Some(via)).await
    }

    fn received_rows(&self) -> Vec<(String, PostRow)> {
        self.opts.db.with_conn(|c| {
            let Ok(mut st) = c.prepare("SELECT id, scope, body, status, outcome FROM posts WHERE status = 'received' ORDER BY received_at") else { return Vec::new() };
            st.query_map([], |r| Ok((r.get(0)?, PostRow { scope: r.get(1)?, body: r.get(2)?, status: r.get(3)?, outcome: r.get(4)? })))
                .map(|rows| rows.filter_map(Result::ok).collect())
                .unwrap_or_default()
        })
    }

    /// Boot recovery: every post still `received` is applied, unless its board already has it.
    pub async fn recover_posts(self: &Arc<Self>) -> usize {
        let rows = self.received_rows();
        for (id, row) in &rows {
            self.recover_one(id, row).await;
        }
        rows.len()
    }

    async fn recover_one(self: &Arc<Self>, id: &str, row: &PostRow) {
        let board = match read_board(self.coord_home(), &row.scope) {
            Ok(b) => b,
            Err(e) => return self.warn("post recovery could not read the board", id, &e),
        };
        if let Some(have) = board.as_ref().and_then(|b| b["entries"].get(id)) {
            let status = if row.scope == PORTFOLIO {
                have["status"].as_str().unwrap_or("delivered").to_owned()
            } else {
                result_of(id, Some(have), 0).status
            };
            return self.finish_post(id, &status, have["outcome"].as_str());
        }
        if board.is_none() && row.scope != PORTFOLIO {
            return self.finish_post(
                id,
                "error",
                Some(&format!("no board for project \"{}\"", row.scope)),
            );
        }
        let Ok(entry) = serde_json::from_str::<Value>(&row.body) else {
            return;
        };
        if let Err(e) = self.run_post(&row.scope, entry).await {
            self.warn("post recovery failed", id, &e);
        }
    }
}
