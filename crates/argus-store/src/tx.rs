//! A transaction: every read and write of the store, with the doc events it will emit once the
//! outermost commit is done.

use std::cell::RefCell;

use argus_core::{AppliedChange, DocsEvent};
use rusqlite::{Connection, OptionalExtension, params};

use crate::StoreError;

/// An open `BEGIN IMMEDIATE` transaction.
pub struct Tx<'a> {
    pub(crate) conn: &'a Connection,
    pub(crate) events: RefCell<Vec<DocsEvent>>,
}

impl<'a> Tx<'a> {
    pub(crate) fn new(conn: &'a Connection) -> Self {
        Tx {
            conn,
            events: RefCell::new(Vec::new()),
        }
    }

    /// The connection, for the tables other crates own (runs, wakes, posts...).
    #[must_use]
    pub fn conn(&self) -> &Connection {
        self.conn
    }

    /// A kv value.
    ///
    /// # Errors
    /// A SQLite failure.
    pub fn kv_get(&self, key: &str) -> Result<Option<String>, StoreError> {
        Ok(self
            .conn
            .query_row("SELECT value FROM kv WHERE key = ?", [key], |r| r.get(0))
            .optional()?)
    }

    /// Sets a kv value.
    ///
    /// # Errors
    /// A SQLite failure.
    pub fn kv_set(&self, key: &str, value: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO kv(key, value) VALUES(?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    /// Atomic +1 of a kv counter; the first call answers 1.
    ///
    /// # Errors
    /// A SQLite failure.
    pub fn next_counter(&self, key: &str) -> Result<i64, StoreError> {
        let text: String = self.conn.query_row(
            "INSERT INTO kv(key, value) VALUES(?, '1') ON CONFLICT(key) DO UPDATE SET value = CAST(value AS INTEGER) + 1 RETURNING value",
            [key],
            |r| r.get(0),
        )?;
        Ok(text.parse().unwrap_or(0))
    }

    /// The highest seq ever handed out, even when every change row was pruned.
    ///
    /// # Errors
    /// A SQLite failure.
    pub fn seq(&self) -> Result<i64, StoreError> {
        let s = self
            .conn
            .query_row(
                "SELECT seq FROM sqlite_sequence WHERE name = 'changes'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        Ok(s.unwrap_or(0))
    }

    pub(crate) fn queue(&self, scope: &str, changes: &[AppliedChange]) {
        if let Some(last) = changes.last() {
            self.events.borrow_mut().push(DocsEvent {
                scope: scope.to_owned(),
                seq: last.seq,
                changes: changes.to_vec(),
            });
        }
    }
}
