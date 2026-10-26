//! The activity log (DESIGN 13.11): an informal record of what the dashboard noticed and did.
//! One line per event in the `activity` table (the last 1000 rows stay); the newest 300 are
//! `activity` docs of `_portfolio`, ids `a-<n>`. `add` never fails its caller.

use std::sync::Arc;

use argus_core::Coll;
use argus_core::clock::{Clock, iso_from_ms, parse_ms};
use argus_core::log::Logger;
use rusqlite::{Connection, params};
use serde_json::json;

use crate::activity_entry::{ActivityEntry, ActivityFilter, ActivityLevel, NewActivity, TEXT_MAX};
use crate::activity_entry::{one_line, short_name, valid_kind};
use crate::activity_notes::Notes;
use crate::{Db, StoreError, Tx};

/// The scope of the activity docs.
pub const SCOPE: &str = argus_core::ids::PORTFOLIO;
/// Rows kept in the table.
pub const KEEP_ROWS: i64 = 1000;
/// Docs kept in the scope.
pub const MIRROR_ROWS: i64 = 300;
const DETAIL_MAX: usize = 160;
const LIST_COLUMNS: &str = "n, at, kind, level, text, project, thread";

/// A line of a service's log that also goes to the activity log.
#[derive(Debug, Clone, Copy)]
pub struct LogNote<'a> {
    /// The event kind, the service's name.
    pub kind: &'a str,
    /// How serious it is.
    pub level: ActivityLevel,
    /// The log message.
    pub message: &'a str,
    /// What went wrong, appended after a colon.
    pub cause: Option<&'a str>,
    /// The project it is about.
    pub project: Option<&'a str>,
}

/// The activity log service.
pub struct ActivityLog {
    db: Arc<Db>,
    clock: Arc<dyn Clock>,
    log: Arc<Logger>,
    notes: Notes,
}

fn entry_of(r: &rusqlite::Row<'_>) -> rusqlite::Result<ActivityEntry> {
    Ok(ActivityEntry {
        n: r.get(0)?,
        at: r.get(1)?,
        kind: r.get(2)?,
        level: ActivityLevel::or_info(&r.get::<_, String>(3)?),
        text: r.get(4)?,
        project: r.get(5)?,
        thread: r.get(6)?,
    })
}

fn newest(conn: &Connection, filter: &ActivityFilter) -> Result<Vec<ActivityEntry>, StoreError> {
    let sql = format!(
        "SELECT {LIST_COLUMNS} FROM activity WHERE (?1 IS NULL OR kind = ?1) AND (?2 IS NULL OR level = ?2) ORDER BY n DESC LIMIT ?3"
    );
    let mut st = conn.prepare_cached(&sql)?;
    let rows = st.query_map(
        params![
            filter.kind,
            filter.level.map(ActivityLevel::as_str),
            filter.limit.clamp(1, KEEP_ROWS)
        ],
        entry_of,
    )?;
    Ok(rows.collect::<Result<_, _>>()?)
}

fn insert(tx: &Tx, entry: &mut ActivityEntry) -> Result<(), StoreError> {
    let conn = tx.conn();
    conn.execute(
        "INSERT INTO activity(at, kind, level, text, project, thread) VALUES(?, ?, ?, ?, ?, ?)",
        params![
            entry.at,
            entry.kind,
            entry.level.as_str(),
            entry.text,
            entry.project,
            entry.thread
        ],
    )?;
    entry.n = conn.last_insert_rowid();
    conn.execute("DELETE FROM activity WHERE n <= ?", [entry.n - KEEP_ROWS])?;
    let mut changes = vec![entry.set_change()];
    if entry.n > MIRROR_ROWS {
        changes.push(argus_core::DocChange::delete(
            Coll::Activity,
            format!("a-{}", entry.n - MIRROR_ROWS),
        ));
    }
    tx.apply(SCOPE, &changes)?;
    Ok(())
}

impl ActivityLog {
    /// A log over the database; `clock` stamps the lines, `log` hears what could not be written.
    #[must_use]
    pub fn new(db: Arc<Db>, clock: Arc<dyn Clock>, log: Arc<Logger>) -> Self {
        ActivityLog {
            db,
            clock,
            log,
            notes: Notes::default(),
        }
    }

    fn entry_for(&self, new: NewActivity) -> Option<ActivityEntry> {
        let text = one_line(&new.text, TEXT_MAX);
        if !valid_kind(&new.kind) || text.is_empty() {
            return None;
        }
        let ms = new
            .at
            .as_deref()
            .and_then(parse_ms)
            .unwrap_or_else(|| self.clock.now_ms());
        Some(ActivityEntry {
            n: 0,
            at: iso_from_ms(ms),
            kind: new.kind,
            level: new.level,
            text,
            project: short_name(new.project.as_deref()),
            thread: short_name(new.thread.as_deref()),
        })
    }

    /// Writes a line and its doc. `None` when the event is not valid or could not be stored; it
    /// never fails its caller.
    pub fn add(&self, new: NewActivity) -> Option<ActivityEntry> {
        let mut entry = self.entry_for(new)?;
        match self.db.tx(|tx| insert(tx, &mut entry)) {
            Ok(()) => Some(entry),
            Err(e) => {
                self.log
                    .warn("activity not recorded", json!({ "err": e.to_string() }));
                None
            }
        }
    }

    /// The newest lines first, narrowed by kind and level.
    ///
    /// # Errors
    /// A SQLite failure.
    pub fn list(&self, filter: &ActivityFilter) -> Result<Vec<ActivityEntry>, StoreError> {
        self.db.with_conn(|c| newest(c, filter))
    }

    /// Writes every doc again: at start, and after a hard resync wiped the scope.
    pub fn mirror(&self) {
        let filter = ActivityFilter {
            limit: MIRROR_ROWS,
            ..ActivityFilter::default()
        };
        let written = self.list(&filter).and_then(|rows| {
            let all = rows.iter().map(ActivityEntry::set_change).collect();
            self.db.docs().replace(SCOPE, &[Coll::Activity], all)
        });
        if let Err(e) = written {
            self.log
                .warn("activity mirror failed", json!({ "err": e.to_string() }));
        }
    }

    /// A warn or error line of a service's log, as `kind`, with the cause after a colon. One
    /// line per message within a minute.
    pub fn note(&self, note: &LogNote<'_>) {
        if !self
            .notes
            .admit(note.kind, note.message, self.clock.now_ms())
        {
            return;
        }
        let cause = note
            .cause
            .map(|d| one_line(d, DETAIL_MAX))
            .filter(|d| !d.is_empty())
            .map_or(String::new(), |d| format!(": {d}"));
        let text = format!("{}{cause}", note.message);
        let line = NewActivity::new(note.kind, note.level, text);
        self.add(line.project(note.project.map(str::to_owned)));
    }
}
