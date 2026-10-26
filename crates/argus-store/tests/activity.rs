//! Port of `activity.test.mjs`: add, the 1000-row cap, the 300-doc mirror in `_portfolio`, the
//! query, the logged lines (one per message a minute) and a log that never fails its caller.

use std::sync::Arc;

use argus_core::clock::{ManualClock, parse_ms};
use argus_core::log::{LOG_MAX_BYTES, Logger};
use argus_core::redact::Redactor;
use argus_core::{Bus, Coll};
use argus_store::{
    ACTIVITY_SCOPE, ActivityEntry, ActivityFilter, ActivityLevel, ActivityLog, Db, KEEP_ROWS,
    LogNote, MIRROR_ROWS, NewActivity,
};
use serde_json::{Value, json};
use tempfile::TempDir;

struct Rig {
    log: ActivityLog,
    db: Arc<Db>,
    clock: ManualClock,
    logger_file: std::path::PathBuf,
    _dir: TempDir,
}

fn rig() -> Rig {
    let dir = TempDir::new().unwrap();
    let db = Arc::new(Db::open(&dir.path().join("t.db"), Bus::new()).unwrap());
    let clock = ManualClock::at(parse_ms("2026-10-25T12:00:00Z").unwrap());
    let file = dir.path().join("server.log");
    let logger = Arc::new(Logger::new(&file, Redactor::default(), LOG_MAX_BYTES));
    let log = ActivityLog::new(Arc::clone(&db), Arc::new(clock.clone()), logger);
    Rig {
        log,
        db,
        clock,
        logger_file: file,
        _dir: dir,
    }
}

fn line(kind: &str, level: ActivityLevel, text: &str) -> NewActivity {
    NewActivity::new(kind, level, text)
}

fn docs(db: &Db) -> Vec<Value> {
    let rows = db.docs().list(ACTIVITY_SCOPE, Coll::Activity).unwrap();
    rows.into_iter().map(|r| r.data).collect()
}

fn numbers(rows: &[ActivityEntry]) -> Vec<i64> {
    rows.iter().map(|r| r.n).collect()
}

fn all(r: &Rig) -> Vec<ActivityEntry> {
    r.log.list(&ActivityFilter::default()).unwrap()
}

#[test]
fn add_writes_a_row_and_its_doc_and_folds_the_text() {
    let r = rig();
    let new = line(
        "process-kill",
        ActivityLevel::Warn,
        "  killed   node 4812\n(vite) ",
    )
    .project(Some("rotp".into()))
    .thread(Some("lead".into()));
    let row = r.log.add(new).unwrap();
    let want = json!({ "id": "a-1", "n": 1, "at": "2026-10-25T12:00:00.000Z", "kind": "process-kill", "level": "warn", "text": "killed node 4812 (vite)", "project": "rotp", "thread": "lead" });
    assert_eq!(row.to_doc(), want);
    let stored =
        r.db.docs()
            .get(ACTIVITY_SCOPE, Coll::Activity, "a-1")
            .unwrap();
    assert_eq!(stored.unwrap().data, want);
}

#[test]
fn bad_input_is_refused_quietly() {
    let r = rig();
    assert!(
        r.log
            .add(line("Bad Kind", ActivityLevel::Info, "x"))
            .is_none()
    );
    assert!(r.log.add(line("ok", ActivityLevel::Info, "   ")).is_none());
    assert!(
        r.log
            .add(line("9lives", ActivityLevel::Info, "x"))
            .is_none()
    );
    assert!(
        r.log
            .add(line(&"k".repeat(41), ActivityLevel::Info, "x"))
            .is_none()
    );
    assert!(
        r.log
            .add(line(&"k".repeat(40), ActivityLevel::Info, "x"))
            .is_some()
    );
    assert_eq!(ActivityLevel::or_info("loud"), ActivityLevel::Info);
}

#[test]
fn a_long_text_is_cut_at_400_characters_with_an_ellipsis() {
    let r = rig();
    let row = r
        .log
        .add(line("k", ActivityLevel::Info, &"w".repeat(900)))
        .unwrap();
    assert_eq!(row.text.chars().count(), 400);
    assert!(row.text.ends_with('\u{2026}'));
}

#[test]
fn list_filters_newest_first_and_an_explicit_time_is_kept() {
    let r = rig();
    r.log
        .add(line("process-kill", ActivityLevel::Warn, "a"))
        .unwrap();
    r.log.add(line("ok", ActivityLevel::Info, "b")).unwrap();
    let at = Some("2026-10-24T00:00:00Z".to_owned());
    r.log
        .add(line("wake-fault", ActivityLevel::Error, "z").at(at))
        .unwrap();
    assert_eq!(numbers(&all(&r)), [3, 2, 1]);
    let by_kind = ActivityFilter {
        kind: Some("wake-fault".into()),
        ..ActivityFilter::default()
    };
    let rows = r.log.list(&by_kind).unwrap();
    assert_eq!(rows[0].text, "z");
    assert_eq!(rows[0].at, "2026-10-24T00:00:00.000Z");
    let by_level = ActivityFilter {
        level: Some(ActivityLevel::Warn),
        ..ActivityFilter::default()
    };
    assert_eq!(numbers(&r.log.list(&by_level).unwrap()), [1]);
    let one = ActivityFilter {
        limit: 1,
        ..ActivityFilter::default()
    };
    assert_eq!(numbers(&r.log.list(&one).unwrap()), [3]);
}

#[test]
fn the_table_keeps_1000_rows_and_the_docs_the_newest_300() {
    let r = rig();
    let extra = 5;
    for i in 0..KEEP_ROWS + extra {
        r.log
            .add(line("k", ActivityLevel::Info, &format!("line {i}")))
            .unwrap();
    }
    let rows = r.log.list(&ActivityFilter {
        limit: 5000,
        ..ActivityFilter::default()
    });
    let rows = rows.unwrap();
    assert_eq!(i64::try_from(rows.len()).unwrap(), KEEP_ROWS);
    assert_eq!(rows.last().unwrap().n, extra + 1);
    let docs = docs(&r.db);
    assert_eq!(i64::try_from(docs.len()).unwrap(), MIRROR_ROWS);
    let lowest = docs.iter().map(|d| d["n"].as_i64().unwrap()).min().unwrap();
    assert_eq!(lowest, KEEP_ROWS + extra - MIRROR_ROWS + 1);
}

#[test]
fn mirror_writes_the_newest_300_again_after_the_scope_was_wiped() {
    let r = rig();
    for i in 0..MIRROR_ROWS + 20 {
        r.log
            .add(line("k", ActivityLevel::Info, &format!("line {i}")))
            .unwrap();
    }
    r.db.docs().clear(ACTIVITY_SCOPE).unwrap();
    assert_eq!(docs(&r.db).len(), 0);
    r.log.mirror();
    let docs = docs(&r.db);
    assert_eq!(i64::try_from(docs.len()).unwrap(), MIRROR_ROWS);
    let top = docs.iter().map(|d| d["n"].as_i64().unwrap()).max().unwrap();
    assert_eq!(top, MIRROR_ROWS + 20);
}

#[test]
fn add_never_fails_its_caller_and_says_so_in_the_log() {
    let r = rig();
    r.db.with_conn(|c| c.execute("DROP TABLE activity", []).unwrap());
    assert!(r.log.add(line("k", ActivityLevel::Info, "x")).is_none());
    let text = std::fs::read_to_string(&r.logger_file).unwrap();
    assert!(text.contains("activity not recorded"), "{text}");
}

fn note<'a>(message: &'a str, cause: Option<&'a str>, project: Option<&'a str>) -> LogNote<'a> {
    LogNote {
        kind: "wake",
        level: ActivityLevel::Warn,
        message,
        cause,
        project,
    }
}

#[test]
fn a_logged_line_is_recorded_once_per_message_a_minute() {
    let r = rig();
    let tick = note("registry tick failed", Some("boom"), None);
    r.log.note(&tick);
    r.clock.advance(30_000);
    r.log.note(&tick);
    let mut mirror = note("wake mirror failed", Some("db busy"), Some("rotp"));
    mirror.level = ActivityLevel::Error;
    r.log.note(&mirror);
    r.clock.advance(31_000);
    r.log.note(&tick);
    let shown: Vec<(String, String, Option<String>)> = all(&r)
        .into_iter()
        .rev()
        .map(|e| (e.level.as_str().to_owned(), e.text, e.project))
        .collect();
    let want = [
        ("warn", "registry tick failed: boom", None),
        ("error", "wake mirror failed: db busy", Some("rotp")),
        ("warn", "registry tick failed: boom", None),
    ];
    let want: Vec<(String, String, Option<String>)> = want
        .into_iter()
        .map(|(l, t, p)| (l.to_owned(), t.to_owned(), p.map(str::to_owned)))
        .collect();
    assert_eq!(shown, want);
}
