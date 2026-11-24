//! K3: the board lock, the refused wait, the damaged board and the old-format board, and
//! `runDefs` filled by normalize.

// test-tag: critical

use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use argus_board::BoardError;
use argus_board::store::{
    BoardDamage, LockWait, board_dir, empty_board, normalize, read_board, try_lock, with_board,
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
async fn with_board_async_gives_up_with_locked_when_the_lock_is_held_and_no_wait_is_left() {
    let home = home_with_board();
    let held = try_lock(&board_dir(home.path(), "lockme"))
        .expect("lock")
        .expect("taken");
    let opts = LockWait {
        wait: Duration::ZERO,
        create: false,
    };
    let result = with_board_async(home.path(), "lockme", opts, |_| -> Result<(), BoardError> {
        panic!("must not run")
    })
    .await;
    drop(held);
    assert!(matches!(result, Err(BoardError::Locked(key)) if key == "lockme"));
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

#[test]
fn a_board_of_nul_bytes_is_a_damaged_board_not_a_parse_error() {
    let home = home_with_board();
    fs::write(
        argus_board::store::board_path(home.path(), "lockme"),
        [0u8; 2048],
    )
    .unwrap();
    let err = read_board(home.path(), "lockme").expect_err("refused");
    assert!(
        matches!(
            &err,
            BoardError::Damaged {
                damage: BoardDamage::AllNul { bytes: 2048 },
                ..
            }
        ),
        "{err}"
    );
    let text = err.to_string();
    assert!(
        text.contains("2048 NUL bytes") && text.contains("board.json"),
        "{text}"
    );
    let locked = with_board(home.path(), "lockme", false, |_| Ok(()));
    assert!(matches!(locked, Err(BoardError::Damaged { .. })));
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
    let backups = backups_named(&dir, "board.v1.");
    assert_eq!(backups.len(), 1, "the unmigrated text is kept once");
}

#[test]
fn a_coordinator_board_is_renamed_on_read_and_its_first_save_keeps_the_old_text() {
    let home = tempfile::tempdir().expect("temp dir");
    let old = json!({
        "schema": 2,
        "project": "lead-old",
        "threads": {"coordinator": {"id": "coordinator", "role": "coordinator"}},
        "items": {"I-1": {"id": "I-1", "assignee": "coordinator", "thread": "coordinator"}},
    });
    let dir = board_dir(home.path(), "lead-old");
    fs::create_dir_all(&dir).expect("dir");
    fs::write(dir.join("board.json"), old.to_string()).expect("write");
    let board = read_board(home.path(), "lead-old")
        .expect("read")
        .expect("a board");
    assert_eq!(board["threads"]["lead"]["role"], json!("lead"));
    assert_eq!(board["items"]["I-1"]["assignee"], json!("lead"));
    with_board(home.path(), "lead-old", false, |_| Ok(())).expect("saved");
    let backups = backups_named(&dir, "board.pre-lead.");
    assert_eq!(backups.len(), 1, "the old text is kept once");
    let again = read_board(home.path(), "lead-old")
        .expect("read")
        .expect("a board");
    assert_eq!(again["threads"]["lead"]["id"], json!("lead"));
}

fn backups_named(dir: &Path, prefix: &str) -> Vec<String> {
    fs::read_dir(dir)
        .expect("list")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with(prefix))
        .collect()
}
