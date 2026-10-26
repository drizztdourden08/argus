//! The doc store inside a transaction: the `docs` mirror with a sha1 per doc, and the `changes`
//! log the streams resume from. A set whose hash is unchanged writes nothing.

use std::collections::{HashMap, HashSet};

use argus_core::clock::{iso_from_ms, now_iso, now_ms};
use argus_core::ids::is_scope;
use argus_core::{AppliedChange, Coll, DocChange, DocOp};
use rusqlite::{OptionalExtension, params};
use serde_json::Value;
use sha1::{Digest, Sha1};

use crate::{StoreError, Tx};

const CHANGES_KEEP_ROWS: i64 = 20_000;
const CHANGES_KEEP_MS: i64 = 7 * 24 * 3600 * 1000;

/// A stored doc.
#[derive(Debug, Clone, PartialEq)]
pub struct DocRow {
    /// Its id.
    pub id: String,
    /// Its JSON.
    pub data: Value,
    /// The seq of its last change.
    pub seq: i64,
}

/// What an apply or replace wrote.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Applied {
    /// The seq of the last change, or the current seq when nothing changed.
    pub seq: i64,
    /// What changed, in order.
    pub changes: Vec<AppliedChange>,
}

/// Limits of a prune.
#[derive(Debug, Clone, Copy)]
pub struct PruneOptions {
    /// Change rows kept.
    pub max_rows: i64,
    /// Oldest change kept, in milliseconds.
    pub max_age_ms: i64,
    /// The time to measure from.
    pub now_ms: i64,
}

impl Default for PruneOptions {
    fn default() -> Self {
        PruneOptions {
            max_rows: CHANGES_KEEP_ROWS,
            max_age_ms: CHANGES_KEEP_MS,
            now_ms: now_ms(),
        }
    }
}

pub(crate) fn check_scope(scope: &str) -> Result<(), StoreError> {
    if is_scope(scope) {
        Ok(())
    } else {
        Err(StoreError::bad(format!("invalid scope \"{scope}\"")))
    }
}

fn sha1_hex(text: &str) -> String {
    argus_core::config::hex(&Sha1::digest(text.as_bytes()))
}

fn parse_data(text: &str) -> Result<Value, StoreError> {
    Ok(serde_json::from_str(text)?)
}

fn coll_of(name: &str) -> Coll {
    name.parse().unwrap_or(Coll::Meta)
}

impl Tx<'_> {
    /// Every doc of a collection, by id.
    ///
    /// # Errors
    /// A bad scope or a SQLite failure.
    pub fn list(&self, scope: &str, coll: Coll) -> Result<Vec<DocRow>, StoreError> {
        check_scope(scope)?;
        let mut st = self.conn.prepare_cached(
            "SELECT id, data, seq FROM docs WHERE scope = ? AND coll = ? ORDER BY id",
        )?;
        let rows = st.query_map(params![scope, coll.as_str()], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get(2)?))
        })?;
        rows.map(|r| {
            let (id, data, seq) = r?;
            Ok(DocRow {
                id,
                data: parse_data(&data)?,
                seq,
            })
        })
        .collect()
    }

    /// One doc.
    ///
    /// # Errors
    /// A bad scope or a SQLite failure.
    pub fn get(&self, scope: &str, coll: Coll, id: &str) -> Result<Option<DocRow>, StoreError> {
        check_scope(scope)?;
        let row: Option<(String, i64)> = self
            .conn
            .query_row(
                "SELECT data, seq FROM docs WHERE scope = ? AND coll = ? AND id = ?",
                params![scope, coll.as_str(), id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        row.map(|(data, seq)| {
            Ok(DocRow {
                id: id.to_owned(),
                data: parse_data(&data)?,
                seq,
            })
        })
        .transpose()
    }

    /// `coll/id` to hash for a scope.
    ///
    /// # Errors
    /// A bad scope or a SQLite failure.
    pub fn hashes(&self, scope: &str) -> Result<HashMap<String, String>, StoreError> {
        check_scope(scope)?;
        let mut st = self
            .conn
            .prepare_cached("SELECT coll, id, hash FROM docs WHERE scope = ?")?;
        let rows = st.query_map([scope], |r| {
            Ok((
                format!("{}/{}", r.get::<_, String>(0)?, r.get::<_, String>(1)?),
                r.get::<_, String>(2)?,
            ))
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    fn stored_hash(&self, scope: &str, coll: Coll, id: &str) -> Result<Option<String>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT hash FROM docs WHERE scope = ? AND coll = ? AND id = ?",
                params![scope, coll.as_str(), id],
                |r| r.get(0),
            )
            .optional()?)
    }

    fn log_change(
        &self,
        scope: &str,
        coll: Coll,
        id: &str,
        op: DocOp,
        at: &str,
    ) -> Result<i64, StoreError> {
        self.conn.execute(
            "INSERT INTO changes(scope, coll, id, op, at) VALUES(?, ?, ?, ?, ?)",
            params![scope, coll.as_str(), id, op.as_str(), at],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    fn apply_delete(
        &self,
        scope: &str,
        c: &DocChange,
    ) -> Result<Option<AppliedChange>, StoreError> {
        if self.stored_hash(scope, c.coll, &c.id)?.is_none() {
            return Ok(None);
        }
        self.conn.execute(
            "DELETE FROM docs WHERE scope = ? AND coll = ? AND id = ?",
            params![scope, c.coll.as_str(), c.id],
        )?;
        let seq = self.log_change(scope, c.coll, &c.id, DocOp::Delete, &now_iso())?;
        Ok(Some(AppliedChange {
            change: DocChange::delete(c.coll, c.id.clone()),
            seq,
        }))
    }

    fn apply_set(&self, scope: &str, c: &DocChange) -> Result<Option<AppliedChange>, StoreError> {
        let data = c.data.as_ref().filter(|d| d.is_object());
        let Some(data) = data else {
            return Err(StoreError::bad(format!(
                "doc {}/{} must be an object",
                c.coll, c.id
            )));
        };
        let text = argus_core::json_text::to_string(data)?;
        let hash = sha1_hex(&text);
        if self.stored_hash(scope, c.coll, &c.id)?.as_deref() == Some(hash.as_str()) {
            return Ok(None);
        }
        let at = now_iso();
        let seq = self.log_change(scope, c.coll, &c.id, DocOp::Set, &at)?;
        self.conn.execute(
            "INSERT INTO docs(scope, coll, id, data, hash, seq, updated_at) VALUES(?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(scope, coll, id) DO UPDATE SET data = excluded.data, hash = excluded.hash, seq = excluded.seq, updated_at = excluded.updated_at",
            params![scope, c.coll.as_str(), c.id, text, hash, seq, at],
        )?;
        Ok(Some(AppliedChange {
            change: DocChange::set(c.coll, c.id.clone(), data.clone()),
            seq,
        }))
    }

    fn apply_inner(
        &self,
        scope: &str,
        changes: &[DocChange],
    ) -> Result<Vec<AppliedChange>, StoreError> {
        let mut applied = Vec::new();
        for c in changes {
            if c.id.is_empty() {
                return Err(StoreError::bad("a change needs a non-empty id"));
            }
            let done = match c.op {
                DocOp::Delete => self.apply_delete(scope, c)?,
                DocOp::Set => self.apply_set(scope, c)?,
            };
            applied.extend(done);
        }
        Ok(applied)
    }

    fn finish(&self, scope: &str, applied: Vec<AppliedChange>) -> Result<Applied, StoreError> {
        let seq = match applied.last() {
            Some(last) => last.seq,
            None => self.seq()?,
        };
        self.queue(scope, &applied);
        Ok(Applied {
            seq,
            changes: applied,
        })
    }

    /// Sets and deletes, skipping sets whose hash is unchanged; one doc event when anything
    /// changed.
    ///
    /// # Errors
    /// A bad scope or change, or a SQLite failure.
    pub fn apply(&self, scope: &str, changes: &[DocChange]) -> Result<Applied, StoreError> {
        check_scope(scope)?;
        let applied = self.apply_inner(scope, changes)?;
        self.finish(scope, applied)
    }

    /// Sets everything in `all` and deletes the rows of `colls` that `all` no longer has.
    ///
    /// # Errors
    /// A bad scope or change, or a SQLite failure.
    pub fn replace(
        &self,
        scope: &str,
        colls: &[Coll],
        all: Vec<DocChange>,
    ) -> Result<Applied, StoreError> {
        check_scope(scope)?;
        let keep: HashSet<String> = all
            .iter()
            .filter(|c| c.op != DocOp::Delete)
            .map(DocChange::key)
            .collect();
        let mut changes = all;
        for coll in colls {
            let mut st = self
                .conn
                .prepare_cached("SELECT id FROM docs WHERE scope = ? AND coll = ?")?;
            let ids: Vec<String> = st
                .query_map(params![scope, coll.as_str()], |r| r.get(0))?
                .collect::<Result<_, _>>()?;
            changes.extend(
                ids.into_iter()
                    .filter(|id| !keep.contains(&format!("{coll}/{id}")))
                    .map(|id| DocChange::delete(*coll, id)),
            );
        }
        let applied = self.apply_inner(scope, &changes)?;
        self.finish(scope, applied)
    }

    /// Forgets a scope and its change history; old resume points fall back to a snapshot.
    ///
    /// # Errors
    /// A bad scope or a SQLite failure.
    pub fn clear(&self, scope: &str) -> Result<(), StoreError> {
        check_scope(scope)?;
        self.conn
            .execute("DELETE FROM docs WHERE scope = ?", [scope])?;
        self.conn
            .execute("DELETE FROM changes WHERE scope = ?", [scope])?;
        self.kv_set(
            &format!("changes:cleared:{scope}"),
            &self.seq()?.to_string(),
        )
    }

    fn kv_num(&self, key: &str) -> Result<i64, StoreError> {
        Ok(self.kv_get(key)?.and_then(|v| v.parse().ok()).unwrap_or(0))
    }

    /// The changes after `since`, one per doc (its last op); `None` when the log no longer
    /// reaches back that far.
    ///
    /// # Errors
    /// A bad scope or a SQLite failure.
    pub fn changes_since(
        &self,
        scope: &str,
        since: i64,
    ) -> Result<Option<Vec<AppliedChange>>, StoreError> {
        check_scope(scope)?;
        if since < 0 || since > self.seq()? {
            return Ok(None);
        }
        let floor = self
            .kv_num("changes:prunedTo")?
            .max(self.kv_num(&format!("changes:cleared:{scope}"))?);
        if since < floor {
            return Ok(None);
        }
        let mut st = self.conn.prepare_cached(
            "SELECT seq, coll, id, op FROM changes WHERE scope = ? AND seq > ? ORDER BY seq",
        )?;
        let rows: Vec<(i64, String, String, String)> = st
            .query_map(params![scope, since], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
            })?
            .collect::<Result<_, _>>()?;
        let mut last: Vec<(i64, String, String, String)> = Vec::new();
        for row in rows {
            last.retain(|(_, c, i, _)| !(c == &row.1 && i == &row.2));
            last.push(row);
        }
        let mut out = Vec::new();
        for (seq, coll, id, op) in last {
            out.extend(self.resume_change(scope, seq, coll_of(&coll), id, &op)?);
        }
        Ok(Some(out))
    }

    fn resume_change(
        &self,
        scope: &str,
        seq: i64,
        coll: Coll,
        id: String,
        op: &str,
    ) -> Result<Option<AppliedChange>, StoreError> {
        if op == "delete" {
            return Ok(Some(AppliedChange {
                change: DocChange::delete(coll, id),
                seq,
            }));
        }
        Ok(self.get(scope, coll, &id)?.map(|row| AppliedChange {
            change: DocChange::set(coll, id, row.data),
            seq,
        }))
    }

    /// Keeps the newest change rows and nothing older than the age limit; answers the rows
    /// removed.
    ///
    /// # Errors
    /// A SQLite failure.
    pub fn prune(&self, o: PruneOptions) -> Result<usize, StoreError> {
        let by_rows = self.seq()? - o.max_rows;
        let cutoff = iso_from_ms(o.now_ms - o.max_age_ms);
        let by_age: Option<i64> =
            self.conn
                .query_row("SELECT MAX(seq) FROM changes WHERE at < ?", [cutoff], |r| {
                    r.get(0)
                })?;
        let up_to = by_rows.max(by_age.unwrap_or(0)).max(0);
        if up_to <= 0 {
            return Ok(0);
        }
        let n = self
            .conn
            .execute("DELETE FROM changes WHERE seq <= ?", [up_to])?;
        if n > 0 {
            let floor = up_to.max(self.kv_num("changes:prunedTo")?);
            self.kv_set("changes:prunedTo", &floor.to_string())?;
        }
        Ok(n)
    }
}
