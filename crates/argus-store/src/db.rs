//! Opening the database (pragmas, migrations, refusal of a newer schema) and running
//! transactions. One connection behind a mutex: the server is the only writer.

use std::fs;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, PoisonError};

use argus_core::{Bus, Coll, DocChange};
use rusqlite::Connection;

use crate::{Applied, DocRow, MIGRATIONS, PruneOptions, Query, StoreError, Tx};

const PRAGMAS: &str = "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000; PRAGMA temp_store=MEMORY;";

/// The database.
pub struct Db {
    conn: Mutex<Connection>,
    bus: Bus,
}

fn migrate(conn: &Connection, migrations: &[&str]) -> Result<(), StoreError> {
    let have: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let known = i64::try_from(migrations.len()).unwrap_or(i64::MAX);
    if have > known {
        return Err(StoreError::Newer {
            have,
            known: migrations.len(),
        });
    }
    if have == known {
        return Ok(());
    }
    conn.execute_batch("BEGIN IMMEDIATE")?;
    let run = || -> Result<(), StoreError> {
        for m in &migrations[usize::try_from(have).unwrap_or(0)..] {
            conn.execute_batch(m)?;
        }
        conn.execute_batch(&format!("PRAGMA user_version = {known}"))?;
        conn.execute_batch("COMMIT")?;
        Ok(())
    };
    run().inspect_err(|_| {
        let _ = conn.execute_batch("ROLLBACK");
    })
}

impl Db {
    /// Opens (creating it and its folder when missing) with the Node migrations. Doc events go
    /// to `bus`.
    ///
    /// # Errors
    /// A newer schema, a failed migration, or an unopenable file.
    pub fn open(file: &Path, bus: Bus) -> Result<Db, StoreError> {
        Self::open_with(file, bus, &MIGRATIONS)
    }

    /// Opens with an explicit migration list (tests).
    ///
    /// # Errors
    /// A newer schema, a failed migration, or an unopenable file.
    pub fn open_with(file: &Path, bus: Bus, migrations: &[&str]) -> Result<Db, StoreError> {
        if let Some(dir) = file.parent() {
            fs::create_dir_all(dir)?;
        }
        let conn = Connection::open(file)?;
        conn.execute_batch(PRAGMAS)?;
        migrate(&conn, migrations)?;
        Ok(Db {
            conn: Mutex::new(conn),
            bus,
        })
    }

    fn lock(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The bus doc events go to.
    #[must_use]
    pub fn bus(&self) -> &Bus {
        &self.bus
    }

    /// Runs `f` in `BEGIN IMMEDIATE` ... `COMMIT`, rolled back when it fails. Doc events fire
    /// after the commit, never after a rollback.
    ///
    /// # Errors
    /// What `f` returns, or `Busy` when the database stays locked.
    pub fn tx<T>(&self, f: impl FnOnce(&Tx) -> Result<T, StoreError>) -> Result<T, StoreError> {
        let events = {
            let conn = self.lock();
            conn.execute_batch("BEGIN IMMEDIATE")?;
            let tx = Tx::new(&conn);
            let out = f(&tx).and_then(|v| {
                conn.execute_batch("COMMIT")
                    .map(|()| v)
                    .map_err(StoreError::from)
            });
            match out {
                Ok(v) => (v, tx.events.take()),
                Err(e) => {
                    let _ = conn.execute_batch("ROLLBACK");
                    return Err(e);
                }
            }
        };
        let (value, fired) = events;
        for e in &fired {
            self.bus.emit_docs(e);
        }
        Ok(value)
    }

    /// The raw connection, outside any transaction (reads, other crates' tables).
    pub fn with_conn<T>(&self, f: impl FnOnce(&Connection) -> T) -> T {
        f(&self.lock())
    }

    /// `PRAGMA user_version`.
    ///
    /// # Errors
    /// A SQLite failure.
    pub fn user_version(&self) -> Result<i64, StoreError> {
        Ok(self
            .lock()
            .query_row("PRAGMA user_version", [], |r| r.get(0))?)
    }

    /// The doc store.
    #[must_use]
    pub fn docs(&self) -> DocStore<'_> {
        DocStore(self)
    }

    /// A kv value.
    ///
    /// # Errors
    /// A SQLite failure.
    pub fn kv_get(&self, key: &str) -> Result<Option<String>, StoreError> {
        Tx::new(&self.lock()).kv_get(key)
    }

    /// Sets a kv value.
    ///
    /// # Errors
    /// A SQLite failure.
    pub fn kv_set(&self, key: &str, value: &str) -> Result<(), StoreError> {
        self.tx(|tx| tx.kv_set(key, value))
    }

    /// Atomic +1 of a kv counter.
    ///
    /// # Errors
    /// A SQLite failure.
    pub fn next_counter(&self, key: &str) -> Result<i64, StoreError> {
        self.tx(|tx| tx.next_counter(key))
    }

    /// Checkpoints the WAL; the connection closes when the `Db` is dropped.
    pub fn checkpoint(&self) {
        let _ = self.lock().execute_batch("PRAGMA wal_checkpoint(TRUNCATE)");
    }
}

/// The doc store; each call is its own transaction (use [`Tx`] to group them).
pub struct DocStore<'a>(&'a Db);

impl DocStore<'_> {
    fn read<T>(&self, f: impl FnOnce(&Tx) -> Result<T, StoreError>) -> Result<T, StoreError> {
        f(&Tx::new(&self.0.lock()))
    }

    /// Every doc of a collection.
    ///
    /// # Errors
    /// A bad scope or a SQLite failure.
    pub fn list(&self, scope: &str, coll: Coll) -> Result<Vec<DocRow>, StoreError> {
        self.read(|tx| tx.list(scope, coll))
    }

    /// One doc.
    ///
    /// # Errors
    /// A bad scope or a SQLite failure.
    pub fn get(&self, scope: &str, coll: Coll, id: &str) -> Result<Option<DocRow>, StoreError> {
        self.read(|tx| tx.get(scope, coll, id))
    }

    /// The docs a query selects.
    ///
    /// # Errors
    /// A bad scope or a SQLite failure.
    pub fn query(&self, scope: &str, coll: Coll, q: &Query) -> Result<Vec<DocRow>, StoreError> {
        Ok(q.run(self.list(scope, coll)?))
    }

    /// `coll/id` to hash.
    ///
    /// # Errors
    /// A bad scope or a SQLite failure.
    pub fn hashes(
        &self,
        scope: &str,
    ) -> Result<std::collections::HashMap<String, String>, StoreError> {
        self.read(|tx| tx.hashes(scope))
    }

    /// See [`Tx::apply`].
    ///
    /// # Errors
    /// A bad scope or change, or a SQLite failure.
    pub fn apply(&self, scope: &str, changes: &[DocChange]) -> Result<Applied, StoreError> {
        self.0.tx(|tx| tx.apply(scope, changes))
    }

    /// See [`Tx::replace`].
    ///
    /// # Errors
    /// A bad scope or change, or a SQLite failure.
    pub fn replace(
        &self,
        scope: &str,
        colls: &[Coll],
        all: Vec<DocChange>,
    ) -> Result<Applied, StoreError> {
        self.0.tx(|tx| tx.replace(scope, colls, all))
    }

    /// See [`Tx::clear`].
    ///
    /// # Errors
    /// A bad scope or a SQLite failure.
    pub fn clear(&self, scope: &str) -> Result<(), StoreError> {
        self.0.tx(|tx| tx.clear(scope))
    }

    /// See [`Tx::changes_since`].
    ///
    /// # Errors
    /// A bad scope or a SQLite failure.
    pub fn changes_since(
        &self,
        scope: &str,
        since: i64,
    ) -> Result<Option<Vec<argus_core::AppliedChange>>, StoreError> {
        self.read(|tx| tx.changes_since(scope, since))
    }

    /// See [`Tx::prune`].
    ///
    /// # Errors
    /// A SQLite failure.
    pub fn prune(&self, o: PruneOptions) -> Result<usize, StoreError> {
        self.0.tx(|tx| tx.prune(o))
    }

    /// The highest seq handed out.
    ///
    /// # Errors
    /// A SQLite failure.
    pub fn seq(&self) -> Result<i64, StoreError> {
        Tx::new(&self.0.lock()).seq()
    }
}
