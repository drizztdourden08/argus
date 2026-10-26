//! Port of `core.db.test.mjs`: migrations, newer-schema refusal, tx, doc store apply, replace, queries, changes, clear, prune, busy.

use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use argus_core::{Bus, Coll, DocChange, DocOp, DocsEvent, Subscription};
use argus_store::rusqlite::Connection;
use argus_store::{Db, MIGRATIONS, PruneOptions, Query, StoreError};
use serde_json::{Value, json};
use tempfile::TempDir;

type Events = Arc<Mutex<Vec<DocsEvent>>>;

fn new_db() -> (TempDir, Db) {
    let dir = TempDir::new().unwrap();
    let db = Db::open(&dir.path().join("t.db"), Bus::new()).unwrap();
    (dir, db)
}

fn listen(db: &Db) -> (Subscription, Events) {
    let got: Events = Arc::default();
    let sink = Arc::clone(&got);
    let sub = db
        .bus()
        .on_docs(move |e| sink.lock().unwrap().push(e.clone()));
    (sub, got)
}

fn count(events: &Events) -> usize {
    events.lock().unwrap().len()
}

fn set(coll: Coll, id: &str, data: Value) -> DocChange {
    DocChange::set(coll, id, data)
}

fn del(coll: Coll, id: &str) -> DocChange {
    DocChange::delete(coll, id)
}

fn alpha_set(db: &Db, id: &str, data: Value) -> i64 {
    db.docs()
        .apply("alpha", &[set(Coll::Items, id, data)])
        .unwrap()
        .seq
}

fn bad_message<T: Debug>(r: Result<T, StoreError>) -> String {
    match r.unwrap_err() {
        StoreError::BadRequest(m) => m,
        other => panic!("expected a bad request, got {other:?}"),
    }
}

fn refused(r: Result<Db, StoreError>) -> StoreError {
    r.err().expect("the open should fail")
}

fn pragma<T: argus_store::rusqlite::types::FromSql>(db: &Db, name: &str) -> T {
    db.with_conn(|c| c.query_row(&format!("PRAGMA {name}"), [], |r| r.get(0)))
        .unwrap()
}

fn migration_count() -> i64 {
    i64::try_from(MIGRATIONS.len()).unwrap()
}

fn change_rows(db: &Db) -> i64 {
    db.with_conn(|c| c.query_row("SELECT COUNT(*) FROM changes", [], |r| r.get(0)))
        .unwrap()
}

fn applied_ids(a: &argus_store::Applied) -> Vec<(String, DocOp)> {
    a.changes
        .iter()
        .map(|c| (c.change.id.clone(), c.change.op))
        .collect()
}

#[test]
fn migrations_set_pragmas_tables_and_user_version() {
    let (_dir, db) = new_db();
    assert_eq!(db.user_version().unwrap(), migration_count());
    let tables = [
        "kv",
        "docs",
        "changes",
        "posts",
        "launch_defs",
        "runs",
        "run_logs",
        "attachments",
    ];
    for t in tables {
        let n: i64 = db.with_conn(|c| {
            c.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
                [t],
                |r| r.get(0),
            )
            .unwrap()
        });
        assert_eq!(n, 1, "{t}");
    }
    assert_eq!(pragma::<String>(&db, "journal_mode"), "wal");
    assert_eq!(pragma::<i64>(&db, "foreign_keys"), 1);
    assert_eq!(pragma::<i64>(&db, "busy_timeout"), 5000);
    assert_eq!(pragma::<i64>(&db, "synchronous"), 1);
}

#[test]
fn reopening_keeps_the_data_and_a_newer_database_refuses_to_open() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("reopen.db");
    let db = Db::open(&file, Bus::new()).unwrap();
    db.kv_set("a", "1").unwrap();
    drop(db);
    let db = Db::open(&file, Bus::new()).unwrap();
    assert_eq!(db.kv_get("a").unwrap().as_deref(), Some("1"));
    drop(db);

    let err = refused(Db::open_with(&file, Bus::new(), &[]));
    assert!(matches!(err, StoreError::Newer { known: 0, .. }));
    let want = format!(
        "database schema {} is newer than this dashboard (0)",
        MIGRATIONS.len()
    );
    assert_eq!(err.to_string(), want);

    let mut more: Vec<&str> = MIGRATIONS.to_vec();
    more.push("CREATE TABLE extra (x)");
    let db = Db::open_with(&file, Bus::new(), &more).unwrap();
    assert_eq!(db.user_version().unwrap(), migration_count() + 1);
}

#[test]
fn a_failing_migration_rolls_back_entirely() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("badmig.db");
    let twice = ["CREATE TABLE a (x)", "CREATE TABLE a (x)"];
    assert!(Db::open_with(&file, Bus::new(), &twice).is_err());
    let db = Db::open_with(&file, Bus::new(), &["CREATE TABLE b (x)"]).unwrap();
    assert_eq!(db.user_version().unwrap(), 1);
}

#[test]
fn tx_commits_rolls_back_on_error_and_reads_its_own_writes() {
    let (_dir, db) = new_db();
    db.tx(|tx| tx.kv_set("x", "1")).unwrap();
    assert_eq!(db.kv_get("x").unwrap().as_deref(), Some("1"));

    let failed = db.tx(|tx| {
        tx.kv_set("y", "1")?;
        Err::<(), _>(StoreError::bad("no"))
    });
    assert_eq!(bad_message(failed), "no");
    assert_eq!(db.kv_get("y").unwrap(), None);
    assert!(
        db.with_conn(Connection::is_autocommit),
        "the failed tx rolled back"
    );

    db.tx(|tx| {
        tx.kv_set("z", "1")?;
        assert_eq!(tx.kv_get("z")?.as_deref(), Some("1"));
        Ok(())
    })
    .unwrap();
    assert_eq!(db.kv_get("z").unwrap().as_deref(), Some("1"));
}

#[test]
fn next_counter_is_atomic_plus_one_per_key() {
    let (_dir, db) = new_db();
    let got: Vec<i64> = ["run:a", "run:a", "run:b", "run:a"]
        .into_iter()
        .map(|k| db.next_counter(k).unwrap())
        .collect();
    assert_eq!(got, [1, 2, 1, 3]);
}

#[test]
fn apply_sets_skips_unchanged_hashes_deletes_and_emits_one_event_per_apply() {
    let (_dir, db) = new_db();
    let (_sub, ev) = listen(&db);
    let docs = db.docs();
    let one = json!({ "id": "I-1", "title": "one" });
    let a = docs
        .apply(
            "alpha",
            &[
                set(Coll::Items, "I-1", one.clone()),
                set(Coll::Items, "I-2", json!({ "id": "I-2" })),
            ],
        )
        .unwrap();
    assert_eq!(a.changes.len(), 2);
    assert!(
        a.changes
            .iter()
            .all(|c| c.change.op == DocOp::Set && c.seq > 0)
    );
    assert_eq!(a.seq, a.changes[1].seq);
    assert_eq!(count(&ev), 1);
    assert_eq!(ev.lock().unwrap()[0].scope, "alpha");
    assert_eq!(ev.lock().unwrap()[0].seq, a.seq);

    let again = docs
        .apply("alpha", &[set(Coll::Items, "I-1", one)])
        .unwrap();
    assert_eq!(again.changes.len(), 0);
    assert_eq!(again.seq, a.seq, "nothing written, seq unchanged");
    assert_eq!(count(&ev), 1, "no event when nothing changed");

    let upd = alpha_set(&db, "I-1", json!({ "id": "I-1", "title": "uno" }));
    let row = docs.get("alpha", Coll::Items, "I-1").unwrap().unwrap();
    assert_eq!(row.data["title"], json!("uno"));
    assert_eq!(row.seq, upd);

    let gone = docs
        .apply(
            "alpha",
            &[del(Coll::Items, "I-2"), del(Coll::Items, "I-404")],
        )
        .unwrap();
    assert_eq!(applied_ids(&gone), [("I-2".to_owned(), DocOp::Delete)]);
    assert!(docs.get("alpha", Coll::Items, "I-2").unwrap().is_none());
    assert_eq!(count(&ev), 3);
}

#[test]
fn apply_keeps_scopes_apart_and_rejects_bad_input_whole() {
    let (_dir, db) = new_db();
    let docs = db.docs();
    docs.apply("alpha", &[set(Coll::Items, "I-1", json!({ "n": 1 }))])
        .unwrap();
    docs.apply("beta", &[set(Coll::Items, "I-1", json!({ "n": 2 }))])
        .unwrap();
    let n = |scope: &str| docs.get(scope, Coll::Items, "I-1").unwrap().unwrap().data["n"].clone();
    assert_eq!(n("alpha"), json!(1));
    assert_eq!(n("beta"), json!(2));

    let err = "nope".parse::<Coll>().unwrap_err();
    assert!(err.to_string().contains("unknown collection"));

    let batch = [
        set(Coll::Items, "I-9", json!({ "n": 9 })),
        set(Coll::Items, "I-8", json!([1])),
    ];
    assert!(bad_message(docs.apply("alpha", &batch)).contains("must be an object"));
    assert!(docs.get("alpha", Coll::Items, "I-9").unwrap().is_none());
    assert!(bad_message(docs.apply("Bad Scope", &[])).contains("invalid scope"));
    let no_id = [set(Coll::Items, "", json!({}))];
    assert!(bad_message(docs.apply("alpha", &no_id)).contains("non-empty id"));
}

#[test]
fn an_event_queued_inside_an_outer_tx_fires_after_commit_and_never_after_a_rollback() {
    let (_dir, db) = new_db();
    let (_sub, ev) = listen(&db);
    let change = [set(Coll::Items, "I-1", json!({ "n": 1 }))];
    let aborted = db.tx(|tx| {
        tx.apply("alpha", &change)?;
        Err::<(), _>(StoreError::bad("abort"))
    });
    assert!(aborted.is_err());
    assert_eq!(count(&ev), 0);
    assert!(
        db.docs()
            .get("alpha", Coll::Items, "I-1")
            .unwrap()
            .is_none()
    );

    db.tx(|tx| {
        tx.apply("alpha", &change)?;
        assert_eq!(count(&ev), 0, "not before the commit");
        Ok(())
    })
    .unwrap();
    assert_eq!(count(&ev), 1);
}

#[test]
fn replace_sets_what_is_given_and_deletes_what_is_absent_from_the_listed_collections_only() {
    let (_dir, db) = new_db();
    let docs = db.docs();
    let n = |k: i64| json!({ "n": k });
    docs.apply(
        "alpha",
        &[
            set(Coll::Items, "I-1", n(1)),
            set(Coll::Items, "I-2", n(2)),
            set(Coll::Tests, "T-1", n(1)),
            set(Coll::Runs, "R-1", n(1)),
        ],
    )
    .unwrap();
    let (_sub, ev) = listen(&db);
    let colls = [Coll::Items, Coll::Tests];
    let given = vec![set(Coll::Items, "I-1", n(1)), set(Coll::Items, "I-3", n(3))];
    let r = docs.replace("alpha", &colls, given).unwrap();
    let mut got: Vec<String> = r
        .changes
        .iter()
        .map(|c| format!("{}:{}", c.change.op.as_str(), c.change.key()))
        .collect();
    got.sort();
    assert_eq!(
        got,
        ["delete:items/I-2", "delete:tests/T-1", "set:items/I-3"]
    );
    assert_eq!(count(&ev), 1, "one event for the whole replace");
    let run = docs.get("alpha", Coll::Runs, "R-1").unwrap().unwrap();
    assert_eq!(
        run.data["n"],
        json!(1),
        "collections not listed are untouched"
    );

    let same = vec![set(Coll::Items, "I-1", n(1)), set(Coll::Items, "I-3", n(3))];
    let none = docs.replace("alpha", &colls, same).unwrap();
    assert_eq!(none.changes.len(), 0);
    assert_eq!(count(&ev), 1);
}

#[test]
fn list_get_and_hashes() {
    let (_dir, db) = new_db();
    let docs = db.docs();
    docs.apply(
        "alpha",
        &[
            set(Coll::Items, "I-2", json!({ "n": 2 })),
            set(Coll::Items, "I-1", json!({ "n": 1 })),
            set(Coll::Tests, "T-1", json!({})),
        ],
    )
    .unwrap();
    let ids: Vec<String> = docs
        .list("alpha", Coll::Items)
        .unwrap()
        .into_iter()
        .map(|r| r.id)
        .collect();
    assert_eq!(ids, ["I-1", "I-2"]);
    assert_eq!(docs.list("alpha", Coll::Agents).unwrap().len(), 0);
    let hashes = docs.hashes("alpha").unwrap();
    let mut keys: Vec<&String> = hashes.keys().collect();
    keys.sort();
    assert_eq!(keys, ["items/I-1", "items/I-2", "tests/T-1"]);
    let h = &hashes["items/I-1"];
    assert!(h.len() == 40 && h.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')));
}

fn query_ids(db: &Db, q: &Value) -> Vec<String> {
    let query = Query::parse(q).unwrap();
    let rows = db.docs().query("alpha", Coll::Items, &query).unwrap();
    rows.into_iter().map(|r| r.id).collect()
}

#[test]
fn query_where_operators_order_by_limit_and_dotted_paths() {
    let (_dir, db) = new_db();
    db.docs()
        .apply(
            "alpha",
            &[
                set(Coll::Items, "I-1", json!({ "status": "open", "n": 3, "tags": ["a", "b"], "who": { "name": "x" }, "title": "alpha beta" })),
                set(Coll::Items, "I-2", json!({ "status": "done", "n": 1, "tags": ["b"], "who": { "name": "y" }, "title": "gamma" })),
                set(Coll::Items, "I-3", json!({ "status": "open", "n": 2, "tags": [], "who": { "name": "z" }, "title": "delta beta" })),
            ],
        )
        .unwrap();
    let cases = [
        (
            json!({ "where": [["status", "==", "open"]] }),
            vec!["I-1", "I-3"],
        ),
        (json!({ "where": [["status", "!=", "open"]] }), vec!["I-2"]),
        (
            json!({ "where": [["n", ">", 1]], "orderBy": ["n", "asc"] }),
            vec!["I-3", "I-1"],
        ),
        (
            json!({ "where": [["n", ">=", 2], ["n", "<", 3]] }),
            vec!["I-3"],
        ),
        (json!({ "where": [["n", "<=", 1]] }), vec!["I-2"]),
        (
            json!({ "where": [["status", "in", ["done", "x"]]] }),
            vec!["I-2"],
        ),
        (json!({ "where": [["tags", "contains", "a"]] }), vec!["I-1"]),
        (
            json!({ "where": [["title", "contains", "beta"]] }),
            vec!["I-1", "I-3"],
        ),
        (json!({ "where": [["who.name", "==", "y"]] }), vec!["I-2"]),
        (
            json!({ "orderBy": ["n", "desc"], "limit": 2 }),
            vec!["I-1", "I-3"],
        ),
        (json!({}), vec!["I-1", "I-2", "I-3"]),
    ];
    for (q, want) in cases {
        assert_eq!(query_ids(&db, &q), want, "{q}");
    }
}

#[test]
fn query_parse_refuses_a_bad_operator_limit_and_where() {
    let cases = [
        (json!({ "where": [["n", "~", 1]] }), "unknown operator"),
        (json!({ "limit": 5000 }), "limit"),
        (json!({ "where": "x" }), "where"),
    ];
    for (q, word) in cases {
        let message = bad_message(Query::parse(&q));
        assert!(message.contains(word), "{q}: {message}");
    }
}

#[test]
fn changes_since_collapses_to_the_last_op_per_doc_and_is_none_when_out_of_range() {
    let (_dir, db) = new_db();
    let docs = db.docs();
    let first = docs
        .apply(
            "alpha",
            &[
                set(Coll::Items, "I-1", json!({ "n": 1 })),
                set(Coll::Items, "I-2", json!({ "n": 1 })),
            ],
        )
        .unwrap();
    docs.apply("beta", &[set(Coll::Items, "B-1", json!({ "n": 1 }))])
        .unwrap();
    alpha_set(&db, "I-1", json!({ "n": 2 }));
    docs.apply("alpha", &[del(Coll::Items, "I-2")]).unwrap();

    let ch = docs
        .changes_since("alpha", first.changes[0].seq)
        .unwrap()
        .unwrap();
    let ops: Vec<String> = ch
        .iter()
        .map(|c| format!("{}:{}", c.change.op.as_str(), c.change.id))
        .collect();
    assert_eq!(ops, ["set:I-1", "delete:I-2"]);
    assert_eq!(ch[0].change.data.as_ref().unwrap()["n"], json!(2));
    assert!(ch.iter().all(|c| c.seq > 0));

    let head = docs.seq().unwrap();
    assert_eq!(docs.changes_since("alpha", head).unwrap(), Some(vec![]));
    assert_eq!(
        docs.changes_since("alpha", head + 5).unwrap(),
        None,
        "a seq from the future"
    );
    assert_eq!(docs.changes_since("alpha", -1).unwrap(), None);
    assert_eq!(docs.changes_since("beta", 0).unwrap().unwrap().len(), 1);
}

#[test]
fn prune_keeps_the_newest_rows_and_a_resume_point_behind_it_falls_back_to_none() {
    let (_dir, db) = new_db();
    let docs = db.docs();
    let seqs: Vec<i64> = (0..10)
        .map(|i| alpha_set(&db, &format!("I-{i}"), json!({ "n": i })))
        .collect();
    let by_rows = PruneOptions {
        max_rows: 4,
        ..PruneOptions::default()
    };
    assert_eq!(docs.prune(by_rows).unwrap(), 6);
    assert_eq!(change_rows(&db), 4);
    assert_eq!(
        docs.changes_since("alpha", seqs[0]).unwrap(),
        None,
        "pruned past that seq"
    );
    let kept = docs.changes_since("alpha", seqs[5]).unwrap().unwrap();
    assert_eq!(
        kept.len(),
        4,
        "the oldest kept row is right after the prune point"
    );
    assert_eq!(docs.seq().unwrap(), seqs[9], "seq never goes backwards");
    assert_eq!(
        docs.list("alpha", Coll::Items).unwrap().len(),
        10,
        "docs are not pruned"
    );

    let by_age = PruneOptions {
        max_age_ms: -1000,
        ..PruneOptions::default()
    };
    assert_eq!(docs.prune(by_age).unwrap(), 4);
    assert_eq!(docs.seq().unwrap(), seqs[9]);
    let after = docs.changes_since("alpha", seqs[9]).unwrap().unwrap();
    assert_eq!(after.len(), 0);
    assert_eq!(alpha_set(&db, "I-new", json!({})), seqs[9] + 1);
}

#[test]
fn clear_forgets_one_scope_and_its_history_only() {
    let (_dir, db) = new_db();
    let docs = db.docs();
    let a = alpha_set(&db, "I-1", json!({ "n": 1 }));
    docs.apply("beta", &[set(Coll::Items, "B-1", json!({ "n": 1 }))])
        .unwrap();
    docs.clear("alpha").unwrap();
    assert_eq!(docs.list("alpha", Coll::Items).unwrap().len(), 0);
    assert_eq!(docs.list("beta", Coll::Items).unwrap().len(), 1);
    assert_eq!(
        docs.changes_since("alpha", a - 1).unwrap(),
        None,
        "old resume points snapshot again"
    );
    assert_eq!(
        docs.changes_since("alpha", docs.seq().unwrap()).unwrap(),
        Some(vec![])
    );
    assert!(docs.changes_since("beta", 0).unwrap().is_some());
}

#[test]
fn a_database_locked_by_another_connection_becomes_busy() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("busy.db");
    let a = Db::open(&file, Bus::new()).unwrap();
    let b = Db::open(&file, Bus::new()).unwrap();
    b.with_conn(|c| c.execute_batch("PRAGMA busy_timeout=50"))
        .unwrap();
    a.with_conn(|c| c.execute_batch("BEGIN IMMEDIATE")).unwrap();
    let locked = b.tx(|_| Ok(()));
    assert!(matches!(locked, Err(StoreError::Busy)), "{locked:?}");
    a.with_conn(|c| c.execute_batch("ROLLBACK")).unwrap();
    b.tx(|tx| tx.kv_set("ok", "1")).unwrap();
}
