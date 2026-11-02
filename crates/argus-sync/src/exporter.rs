//! Board to docs. `export_docs` turns a board into page docs and `replace` diffs them by hash:
//! new or changed docs are set, docs the board no longer has are deleted, in one transaction
//! and one bus event, and nothing at all when the store already has it all. That is also why a
//! board the server saved itself causes no echo when the watcher reads it.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use argus_board::Board;
use argus_board::sync::{PAGE_COLLECTIONS, export_docs};
use argus_core::clock::now_iso;
use argus_core::ids::PORTFOLIO;
use argus_core::{ApiError, Coll, DocChange};
use argus_store::rusqlite::params;
use argus_store::{StoreError, Tx};
use serde_json::{Value, json};
use sha1::{Digest, Sha1};

use crate::read::read_board_file;
use crate::state::State;

const RETRY_LATER: Duration = Duration::from_secs(2);
const FOREIGN_META: [&str; 2] = ["procs", "worktrees"];
const PROJECT_FOREIGN_META: [&str; 1] = ["worktrees"];

/// What an export wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Exported {
    pub seq: i64,
    pub changes: usize,
    pub docs: usize,
}

/// Where the page finds a kept proof image.
pub(crate) fn proof_url(key: &str, file: &str) -> String {
    let name = file.rsplit(['/', '\\']).next().unwrap_or(file);
    format!("/files/{key}/proofs/{name}")
}

fn colls(project: bool) -> Vec<Coll> {
    let mut out = PAGE_COLLECTIONS.to_vec();
    out.extend([Coll::Meta, Coll::Launches]);
    if !project {
        out.extend([Coll::Cards, Coll::Deps]);
    }
    out
}

fn sha1_hex(text: &str) -> String {
    argus_core::config::hex(&Sha1::digest(text.as_bytes()))
}

fn stored_defs(tx: &Tx, scope: &str) -> Result<HashMap<String, String>, StoreError> {
    let mut st = tx
        .conn()
        .prepare_cached("SELECT name, hash FROM launch_defs WHERE scope = ?")?;
    let rows = st.query_map([scope], |r| Ok((r.get(0)?, r.get(1)?)))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

fn replace_launch_defs(tx: &Tx, scope: &str, defs: Option<&Value>) -> Result<(), StoreError> {
    let mut have = stored_defs(tx, scope)?;
    let at = now_iso();
    for (name, def) in defs.and_then(Value::as_object).into_iter().flatten() {
        let mut full = def.clone();
        if let Some(o) = full.as_object_mut() {
            o.insert("name".into(), json!(name));
        }
        let text = argus_core::json_text::to_string(&full)?;
        let hash = sha1_hex(&text);
        if have.remove(name).as_deref() != Some(hash.as_str()) {
            tx.conn().execute(
                "INSERT INTO launch_defs(scope, name, def, hash, updated_at) VALUES(?, ?, ?, ?, ?) ON CONFLICT(scope, name) DO UPDATE SET def = excluded.def, hash = excluded.hash, updated_at = excluded.updated_at",
                params![scope, name, text, hash, at],
            )?;
        }
    }
    for name in have.keys() {
        tx.conn().execute(
            "DELETE FROM launch_defs WHERE scope = ? AND name = ?",
            params![scope, name],
        )?;
    }
    Ok(())
}

fn keep_foreign(
    tx: &Tx,
    key: &str,
    project: bool,
    docs: &mut Vec<DocChange>,
) -> Result<(), StoreError> {
    let ids: &[&str] = if project {
        &PROJECT_FOREIGN_META
    } else {
        &FOREIGN_META
    };
    for id in ids {
        if let Some(kept) = tx.get(key, Coll::Meta, id)? {
            docs.push(DocChange::set(Coll::Meta, *id, kept.data));
        }
    }
    Ok(())
}

impl State {
    /// Exports a board already read or just saved, and keeps it as the last good board.
    pub fn export_board(self: &Arc<Self>, key: &str, board: Board) -> Result<Exported, ApiError> {
        let project = key != PORTFOLIO;
        let mut docs = export_docs(&board, &|p| {
            proof_url(key, p["file"].as_str().unwrap_or_default())
        });
        if project {
            self.hooks.overlaid(key, &mut docs);
        }
        let count = docs.len();
        let applied = self.opts.db.tx(|tx| {
            keep_foreign(tx, key, project, &mut docs)?;
            let out = tx.replace(key, &colls(project), docs)?;
            if project {
                replace_launch_defs(tx, key, board.get("runDefs"))?;
            }
            Ok(out)
        })?;
        self.keep_board(key, board);
        if project {
            self.announce(key);
            if !applied.changes.is_empty() {
                self.schedule_portfolio();
            }
        }
        Ok(Exported {
            seq: applied.seq,
            changes: applied.changes.len(),
            docs: count,
        })
    }

    /// Reads the board file and exports it. A board that cannot be read keeps its last good
    /// docs; the error is counted and the read retried a little later.
    pub async fn refresh(self: &Arc<Self>, key: &str) -> Result<Option<Exported>, ApiError> {
        if self.stopped() {
            return Ok(None);
        }
        match read_board_file(self.coord_home(), key).await {
            Ok(Some(board)) if !self.stopped() => self.export_board(key, board).map(Some),
            Ok(_) => Ok(None),
            Err(e) => {
                self.errors.fetch_add(1, Ordering::SeqCst);
                self.warn("board unreadable; keeping the last good copy", key, &e);
                self.retry_later(key);
                Ok(None)
            }
        }
    }

    fn retry_later(self: &Arc<Self>, key: &str) {
        let me = Arc::clone(self);
        let key = key.to_owned();
        tokio::spawn(async move {
            tokio::time::sleep(RETRY_LATER).await;
            if !me.stopped() {
                me.export_key(&key);
            }
        });
    }

    /// Exports a key on its queue in the background.
    pub fn export_key(self: &Arc<Self>, key: &str) {
        let me = Arc::clone(self);
        let key = key.to_owned();
        tokio::spawn(async move { me.export_now(&key).await });
    }

    /// Exports a key on its queue; failures are logged.
    pub async fn export_now(self: &Arc<Self>, key: &str) -> Option<Exported> {
        if self.stopped() {
            return None;
        }
        let out = if key == PORTFOLIO {
            self.refresh_portfolio_queued().await
        } else {
            let me = Arc::clone(self);
            self.opts
                .queue
                .run(key, async move { me.refresh(key).await })
                .await
        };
        out.unwrap_or_else(|e| {
            self.warn("board export failed", key, &e);
            None
        })
    }
}
