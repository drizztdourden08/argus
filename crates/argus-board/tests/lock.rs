//! The lock, the save and the load of a board: `tryLock`, `withBoardAsync` (timeout and a lock
//! released while waiting, without blocking the runtime), the rename retry of the save on Windows
//! sharing errors, and `runDefs` filled by normalize.

use std::fs;
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant, SystemTime};

use argus_board::BoardError;
use argus_board::store::{
    LockWait, board_dir, empty_board, normalize, read_board, save_board_with, try_lock, with_board,
    with_board_async,
};
use serde_json::json;
use tempfile::TempDir;

fn home_with_board() -> TempDir {
    let home = tempfile::tempdir().expect("temp dir");
    with_board(home.path(), "lockme", true, |b| {
        b["name"] = json!("Lock me");
        Ok(())
    })
    .expect("create the board");
    home
}

fn lock_file(home: &Path) -> std::path::PathBuf {
    board_dir(home, "lockme").join("board.lock")
}

fn name_of(home: &Path, key: &str) -> String {
    let board = read_board(home, key).expect("read").expect("a board");
    board["name"].as_str().unwrap_or_default().to_owned()
}

#[test]
fn try_lock_takes_the_lock_once_refuses_while_held_and_clears_a_stale_lock() {
    let home = home_with_board();
    let dir = board_dir(home.path(), "lockme");
    let lock = try_lock(&dir).expect("lock").expect("taken");
    assert!(try_lock(&dir).expect("lock").is_none());
    drop(lock);
    assert!(!lock_file(home.path()).exists());
    fs::write(lock_file(home.path()), "").expect("write a stale lock");
    let old = SystemTime::now() - Duration::from_secs(20);
    fs::File::options()
        .write(true)
        .open(lock_file(home.path()))
        .expect("open")
        .set_modified(old)
        .expect("age the lock");
    let again = try_lock(&dir).expect("lock");
    assert!(again.is_some(), "a lock 20 s old belongs to a crashed call");
}

#[tokio::test]
async fn with_board_async_times_out_with_locked_and_keeps_the_runtime_free() {
    let home = home_with_board();
    let held = try_lock(&board_dir(home.path(), "lockme"))
        .expect("lock")
        .expect("taken");
    let ticks = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&ticks);
    let ticker = tokio::spawn(async move {
        let mut every = tokio::time::interval(Duration::from_millis(20));
        loop {
            every.tick().await;
            counter.fetch_add(1, Ordering::SeqCst);
        }
    });
    let started = Instant::now();
    let opts = LockWait {
        wait: Duration::from_millis(300),
        create: false,
    };
    let result = with_board_async(home.path(), "lockme", opts, |_| -> Result<(), BoardError> {
        panic!("must not run")
    })
    .await;
    ticker.abort();
    drop(held);
    assert!(matches!(result, Err(BoardError::Locked(key)) if key == "lockme"));
    assert!(
        started.elapsed() >= Duration::from_millis(280),
        "waited for the lock"
    );
    assert!(
        ticks.load(Ordering::SeqCst) >= 5,
        "the runtime ran while waiting"
    );
}

#[tokio::test]
async fn with_board_async_runs_once_the_lock_is_released_and_saves() {
    let home = home_with_board();
    let held = try_lock(&board_dir(home.path(), "lockme"))
        .expect("lock")
        .expect("taken");
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(150)).await;
        drop(held);
    });
    let opts = LockWait {
        wait: Duration::from_secs(3),
        create: false,
    };
    let out = with_board_async(home.path(), "lockme", opts, |b| {
        b["name"] = json!("Async");
        Ok("done")
    })
    .await
    .expect("applied");
    assert_eq!(out, "done");
    assert_eq!(name_of(home.path(), "lockme"), "Async");
    assert!(!lock_file(home.path()).exists());
}

#[tokio::test]
async fn with_board_async_refuses_a_missing_board_unless_create_is_set() {
    let home = home_with_board();
    let missing = with_board_async(home.path(), "nothere", LockWait::default(), |_| Ok(())).await;
    assert!(matches!(missing, Err(BoardError::Missing(key)) if key == "nothere"));
    let opts = LockWait {
        create: true,
        ..LockWait::default()
    };
    with_board_async(home.path(), "created", opts, |b| {
        b["name"] = json!("New");
        Ok(())
    })
    .await
    .expect("created");
    assert_eq!(name_of(home.path(), "created"), "New");
}

fn busy() -> io::Error {
    io::Error::from(io::ErrorKind::PermissionDenied)
}

#[test]
fn save_board_retries_a_rename_that_windows_refuses_then_succeeds() {
    let home = home_with_board();
    let mut board = read_board(home.path(), "lockme")
        .expect("read")
        .expect("a board");
    board["name"] = json!("Retried");
    let calls = AtomicUsize::new(0);
    save_board_with(home.path(), &board, &|from, to| {
        if calls.fetch_add(1, Ordering::SeqCst) < 2 {
            Err(busy())
        } else {
            fs::rename(from, to)
        }
    })
    .expect("saved");
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    assert_eq!(name_of(home.path(), "lockme"), "Retried");
}

#[test]
fn save_board_gives_up_after_8_attempts_and_leaves_no_temp_file() {
    let home = home_with_board();
    let mut board = read_board(home.path(), "lockme")
        .expect("read")
        .expect("a board");
    board["name"] = json!("Never");
    let calls = AtomicUsize::new(0);
    let result = save_board_with(home.path(), &board, &|_, _| {
        calls.fetch_add(1, Ordering::SeqCst);
        Err(busy())
    });
    assert!(
        matches!(&result, Err(BoardError::Io(e)) if e.kind() == io::ErrorKind::PermissionDenied)
    );
    assert_eq!(calls.load(Ordering::SeqCst), 8);
    let leftovers: Vec<_> = fs::read_dir(board_dir(home.path(), "lockme"))
        .expect("list")
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
        .collect();
    assert!(leftovers.is_empty());
    assert_eq!(name_of(home.path(), "lockme"), "Lock me");
}

#[test]
fn normalize_fills_run_defs_and_a_new_board_has_them() {
    let mut board = json!({"schema": 2, "project": "plain", "threads": {}});
    normalize(&mut board);
    assert_eq!(board["runDefs"], json!({}));
    assert_eq!(empty_board("x")["runDefs"], json!({}));
}

#[test]
fn a_board_still_on_schema_1_is_migrated_and_its_first_save_keeps_the_old_text() {
    let home = tempfile::tempdir().expect("temp dir");
    let old = json!({
        "project": "old",
        "counters": {"I": 1},
        "threads": {"ui": {"id": "ui", "registeredAt": "2026-01-01T00:00:00.000Z"}},
        "items": {"I-1": {"id": "I-1", "createdAt": "2026-01-02T00:00:00.000Z", "thread": "ui"}},
        "log": {"ui": {"subject": "ui", "entries": [{"text": "a"}, {"text": "b"}]}},
    });
    let dir = board_dir(home.path(), "old");
    fs::create_dir_all(&dir).expect("dir");
    fs::write(
        dir.join("board.json"),
        serde_json::to_string(&old).expect("json"),
    )
    .expect("write");
    let board = read_board(home.path(), "old")
        .expect("read")
        .expect("a board");
    assert_eq!(board["schema"], json!(2));
    assert_eq!(board["items"]["I-1"]["rank"], json!(1024));
    assert_eq!(board["items"]["I-1"]["rev"], json!(1));
    assert_eq!(board["log"]["ui"]["entries"][1]["seq"], json!(2));
    assert_eq!(board["counters"]["log:ui"], json!(2));
    with_board(home.path(), "old", false, |_| Ok(())).expect("saved");
    let backups: Vec<_> = fs::read_dir(&dir)
        .expect("list")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("board.v1."))
        .collect();
    assert_eq!(backups.len(), 1, "the unmigrated text is kept once");
}
