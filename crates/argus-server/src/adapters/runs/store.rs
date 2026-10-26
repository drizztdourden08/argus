//! The run rows over SQLite: the Node `runs` table, row for row, with the semantics of
//! [`argus_launchers::MemoryStore`].

use std::sync::Arc;

use argus_core::log::Logger;
use argus_launchers::{ListFilter, MemoryStore, NewRun, RunPatch, RunRow, RunStore};
use argus_store::Db;
use argus_store::rusqlite::{Params, params, params_from_iter};
use serde_json::json;

use super::patch::assignments;
use super::rows::{COLUMNS, run_of};

const LIVE: &str = "('queued','running')";

/// The run service's storage: rows, log lines and attachment rows in the dashboard database.
pub struct SqliteRuns {
    pub(super) db: Arc<Db>,
    pub(super) log: Arc<Logger>,
    instance: String,
}

impl SqliteRuns {
    /// A store over the database; the rows are stamped with this run's instance id.
    #[must_use]
    pub fn new(db: Arc<Db>, log: Arc<Logger>) -> Self {
        let instance = db
            .kv_get("instance")
            .ok()
            .flatten()
            .unwrap_or_else(|| "local".to_owned());
        SqliteRuns { db, log, instance }
    }

    pub(super) fn failed(&self, what: &str, err: &dyn std::fmt::Display) {
        self.log
            .warn(what, json!({ "err": err.to_string() }));
    }

    fn query(&self, clause: &str, args: impl Params) -> Vec<RunRow> {
        let sql = format!("SELECT {COLUMNS} FROM runs {clause}");
        self.db.with_conn(|c| {
            let Ok(mut statement) = c.prepare_cached(&sql) else {
                return Vec::new();
            };
            statement
                .query_map(args, run_of)
                .map(|rows| rows.flatten().collect())
                .unwrap_or_default()
        })
    }

    fn pairs(&self, sql: &str) -> Vec<(String, String)> {
        self.db.with_conn(|c| {
            let Ok(mut statement) = c.prepare_cached(sql) else {
                return Vec::new();
            };
            statement
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
                .map(|rows| rows.flatten().collect())
                .unwrap_or_default()
        })
    }

    fn insert_row(&self, new: &NewRun) -> Result<(), argus_store::StoreError> {
        self.db.tx(|tx| {
            tx.conn().execute(
                "INSERT INTO runs (id, scope, rid, name, kind, status, cmd, cwd, visible, origin, \"by\", agent, reason, queued_at, instance) VALUES (?, ?, ?, ?, ?, 'queued', ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    format!("{}:{}", new.scope, new.rid),
                    new.scope,
                    new.rid,
                    new.name,
                    new.kind.as_str(),
                    new.cmd,
                    new.cwd,
                    new.visible,
                    new.origin.as_str(),
                    new.by,
                    new.agent,
                    new.reason,
                    new.queued_at,
                    self.instance,
                ],
            )?;
            Ok(())
        })
    }
}

fn where_of(scope: Option<&str>, filter: &ListFilter) -> (String, Vec<String>) {
    let mut wheres = Vec::new();
    let mut args = Vec::new();
    if let Some(scope) = scope {
        wheres.push("scope = ?".to_owned());
        args.push(scope.to_owned());
    }
    if !filter.statuses.is_empty() {
        let marks = vec!["?"; filter.statuses.len()].join(",");
        wheres.push(format!("status IN ({marks})"));
        args.extend(filter.statuses.iter().cloned());
    }
    if let Some(name) = &filter.name {
        wheres.push("name = ?".to_owned());
        args.push(name.clone());
    }
    let clause = if wheres.is_empty() {
        String::new()
    } else {
        format!("WHERE {} ", wheres.join(" AND "))
    };
    (clause, args)
}

impl RunStore for SqliteRuns {
    fn next_number(&self, scope: &str) -> u64 {
        match self.db.next_counter(&format!("run:{scope}")) {
            Ok(n) => u64::try_from(n).unwrap_or(1),
            Err(e) => {
                self.failed("run counter failed", &e);
                1
            }
        }
    }

    fn insert(&self, new: NewRun) -> RunRow {
        match self.insert_row(&new) {
            Ok(()) => self.get(&new.scope, &new.rid),
            Err(e) => {
                self.failed("run row not stored", &e);
                None
            }
        }
        .unwrap_or_else(|| MemoryStore::new().insert(new))
    }

    fn get(&self, scope: &str, rid: &str) -> Option<RunRow> {
        self.query("WHERE scope = ? AND rid = ?", params![scope, rid])
            .pop()
    }

    fn update(&self, scope: &str, rid: &str, patch: &RunPatch) -> Option<RunRow> {
        let sets = assignments(patch);
        if !sets.is_empty() {
            let columns: Vec<String> = sets.iter().map(|(c, _)| format!("{c} = ?")).collect();
            let sql = format!("UPDATE runs SET {} WHERE id = ?", columns.join(", "));
            let mut values: Vec<_> = sets.into_iter().map(|(_, v)| v).collect();
            values.push(format!("{scope}:{rid}").into());
            let done = self.db.tx(|tx| {
                tx.conn().execute(&sql, params_from_iter(values))?;
                Ok(())
            });
            if let Err(e) = done {
                self.failed("run row not updated", &e);
            }
        }
        self.get(scope, rid)
    }

    fn list(&self, scope: Option<&str>, filter: &ListFilter) -> Vec<RunRow> {
        let (clause, args) = where_of(scope, filter);
        let order = format!(
            "{clause}ORDER BY queued_at DESC, rowid DESC LIMIT {}",
            filter.row_limit()
        );
        self.query(&order, params_from_iter(args))
    }

    fn live_by_name(&self, scope: &str, name: &str) -> Vec<RunRow> {
        let clause = format!("WHERE scope = ? AND name = ? AND status IN {LIVE} ORDER BY rowid DESC");
        self.query(&clause, params![scope, name])
    }

    fn live_rows(&self) -> Vec<RunRow> {
        self.query(&format!("WHERE status IN {LIVE} ORDER BY rowid"), [])
    }

    fn queued_rows(&self) -> Vec<RunRow> {
        self.query(
            "WHERE status = 'queued' ORDER BY queued_at, rowid",
            [],
        )
    }

    fn groups(&self) -> Vec<(String, String)> {
        self.pairs("SELECT DISTINCT scope, name FROM runs")
    }

    fn scopes(&self) -> Vec<String> {
        self.db.with_conn(|c| {
            let Ok(mut statement) = c.prepare_cached("SELECT DISTINCT scope FROM runs") else {
                return Vec::new();
            };
            statement
                .query_map([], |r| r.get(0))
                .map(|rows| rows.flatten().collect())
                .unwrap_or_default()
        })
    }

    fn finished(&self, scope: &str, name: &str) -> Vec<RunRow> {
        let clause = format!(
            "WHERE scope = ? AND name = ? AND status NOT IN {LIVE} ORDER BY queued_at DESC, rowid DESC"
        );
        self.query(&clause, params![scope, name])
    }

    fn of_scope(&self, scope: &str) -> Vec<RunRow> {
        self.query("WHERE scope = ? ORDER BY rowid", params![scope])
    }

    fn remove(&self, scope: &str, rid: &str) {
        let done = self.db.tx(|tx| {
            let c = tx.conn();
            c.execute(
                "DELETE FROM run_logs WHERE run_id = ?",
                params![format!("{scope}:{rid}")],
            )?;
            c.execute(
                "DELETE FROM attachments WHERE scope = ? AND owner_kind = 'run' AND owner_id = ?",
                params![scope, rid],
            )?;
            c.execute("DELETE FROM runs WHERE id = ?", params![format!("{scope}:{rid}")])?;
            Ok(())
        });
        if let Err(e) = done {
            self.failed("run not removed", &e);
        }
    }
}
