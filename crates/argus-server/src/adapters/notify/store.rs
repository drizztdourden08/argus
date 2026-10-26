//! The `notifications` table (Node migration 6), its docs `n-<n>` in `_portfolio`, and the
//! markers of the automatic pings in kv `notify:seen`.

use std::sync::Arc;

use argus_core::ids::PORTFOLIO;
use argus_core::{Coll, DocChange, StorageError};
use argus_notify::{By, Filter, KEEP_ROWS, Level, NewNotification, Notification, NotifyStore};
use argus_store::rusqlite::{Connection, Row, params};
use argus_store::{Db, StoreError, Tx};
use serde_json::{Value, json};

const SEEN_KEY: &str = "notify:seen";
const COLUMNS: &str = "n, at, level, title, text, project, thread, by, toast, url, why";

/// Keeps the notifications in the database.
pub struct NotifyDocs {
    db: Arc<Db>,
}

impl NotifyDocs {
    /// The notifications of a database.
    #[must_use]
    pub fn new(db: Arc<Db>) -> Self {
        NotifyDocs { db }
    }
}

fn storage(e: &StoreError) -> StorageError {
    StorageError(e.to_string())
}

fn doc_of(r: &Row<'_>) -> argus_store::rusqlite::Result<Value> {
    let n: i64 = r.get(0)?;
    Ok(json!({
        "id": format!("n-{n}"),
        "n": n,
        "at": r.get::<_, String>(1)?,
        "level": r.get::<_, String>(2)?,
        "title": r.get::<_, String>(3)?,
        "text": r.get::<_, String>(4)?,
        "project": r.get::<_, Option<String>>(5)?,
        "thread": r.get::<_, Option<String>>(6)?,
        "by": r.get::<_, String>(7)?,
        "toast": r.get::<_, String>(8)?,
        "url": r.get::<_, Option<String>>(9)?,
        "why": r.get::<_, Option<String>>(10)?,
    }))
}

fn newest(conn: &Connection, filter: &Filter) -> Result<Vec<Value>, StoreError> {
    let sql = format!(
        "SELECT {COLUMNS} FROM notifications WHERE (?1 IS NULL OR level = ?1) AND (?2 IS NULL OR project = ?2) ORDER BY n DESC LIMIT ?3"
    );
    let limit = i64::try_from(filter.limit).unwrap_or(i64::MAX);
    let mut st = conn.prepare_cached(&sql)?;
    let level = filter.level.map(Level::as_str);
    let rows = st.query_map(params![level, filter.project, limit], doc_of)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

fn mirror_in(tx: &Tx) -> Result<(), StoreError> {
    let all = Filter {
        limit: KEEP_ROWS,
        ..Filter::default()
    };
    let changes = newest(tx.conn(), &all)?
        .into_iter()
        .map(|d| {
            let id = d["id"].as_str().unwrap_or_default().to_owned();
            DocChange::set(Coll::Notifications, id, d)
        })
        .collect();
    tx.replace(PORTFOLIO, &[Coll::Notifications], changes)?;
    Ok(())
}

fn by_name(by: By) -> &'static str {
    match by {
        By::Thread => "thread",
        By::Overlord => "overlord",
        By::Auto => "auto",
        By::Page => "page",
    }
}

fn insert_in(tx: &Tx, new: &NewNotification) -> Result<i64, StoreError> {
    let m = &new.message;
    let conn = tx.conn();
    conn.execute(
        "INSERT INTO notifications(at, level, title, text, project, thread, by, toast, url, why) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            new.at,
            m.level.as_str(),
            m.title,
            m.text,
            m.project,
            m.thread,
            by_name(m.by),
            new.toast.as_str(),
            m.url,
            new.why
        ],
    )?;
    let n = conn.last_insert_rowid();
    let keep = i64::try_from(KEEP_ROWS).unwrap_or(i64::MAX);
    conn.execute("DELETE FROM notifications WHERE n <= ?", [n - keep])?;
    mirror_in(tx)?;
    Ok(n)
}

impl NotifyStore for NotifyDocs {
    fn insert(&self, new: NewNotification) -> Result<Notification, StorageError> {
        let n = self
            .db
            .tx(|tx| insert_in(tx, &new))
            .map_err(|e| storage(&e))?;
        let n = u64::try_from(n).map_err(|e| StorageError(e.to_string()))?;
        Ok(Notification::from_new(n, new))
    }

    fn list(&self, filter: &Filter) -> Result<Vec<Notification>, StorageError> {
        let docs = self
            .db
            .with_conn(|c| newest(c, filter))
            .map_err(|e| storage(&e))?;
        docs.into_iter()
            .map(|d| serde_json::from_value(d).map_err(|e| StorageError(e.to_string())))
            .collect()
    }

    fn mirror(&self) -> Result<(), StorageError> {
        self.db.tx(mirror_in).map_err(|e| storage(&e))
    }

    fn seen_load(&self) -> Option<String> {
        self.db.kv_get(SEEN_KEY).ok().flatten()
    }

    fn seen_save(&self, json: &str) -> Result<(), StorageError> {
        self.db.kv_set(SEEN_KEY, json).map_err(|e| storage(&e))
    }
}
