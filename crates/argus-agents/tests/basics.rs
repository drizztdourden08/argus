//! The small building blocks: ISO time, one-line clips, edit lists, config reading, the file
//! system, and the choice of the thread to wake.

use argus_agents::fakes::{board_json, thread_record};
use argus_agents::iso::{format_iso, parse_iso};
use argus_agents::record::{Patch, Patches, Record, claude_id_of, is_live, stamp};
use argus_agents::text::clip;
use argus_agents::{AgentsConfig, Board, FileSystem, ReconcilerOptions, StdFs};
use serde_json::{Value, json};
use std::fs;

fn record(value: &Value) -> Record {
    value.as_object().cloned().unwrap()
}

#[test]
fn iso_text_round_trips_with_milliseconds() {
    let ms = 1_792_756_800_123;
    assert_eq!(format_iso(ms), "2026-10-23T12:00:00.123Z");
    assert_eq!(parse_iso("2026-10-23T12:00:00.123Z"), Some(ms));
    assert_eq!(parse_iso("2026-10-23T14:00:00.123+02:00"), Some(ms));
    assert_eq!(parse_iso("not a time"), None);
}

#[test]
fn clip_puts_the_text_on_one_line_and_cuts_it() {
    assert_eq!(clip("  git\n  status  ", 80), "git status");
    assert_eq!(clip("abcdef", 3), "abc");
}

#[test]
fn a_patch_sets_and_removes_fields_and_a_later_change_wins() {
    let mut agent = record(&json!({ "a": 1, "b": 2, "c": 3 }));
    let mut patch = Patch::new().with("a", 9).without("b");
    patch.merge(Patch::new().with("a", 10).with("d", "new"));
    patch.apply_to(&mut agent);
    assert_eq!(Value::Object(agent), json!({ "a": 10, "c": 3, "d": "new" }));
}

#[test]
fn patches_show_the_record_as_it_will_be_without_touching_it() {
    let agent = record(&json!({ "id": "A-1", "status": "working", "orphanSince": "x" }));
    let mut patches = Patches::default();
    assert!(patches.is_empty());
    patches.add("A-1", Patch::new().with("status", "stopped"));
    patches.add("A-1", Patch::new().without("orphanSince"));
    let view = patches.view(&agent);
    assert_eq!(view["status"], json!("stopped"));
    assert!(!view.contains_key("orphanSince"));
    assert_eq!(agent["status"], json!("working"));
    assert_eq!(patches.iter().count(), 1);
}

#[test]
fn a_stamp_counts_the_revision_and_names_who_and_when() {
    let mut agent = record(&json!({ "id": "A-1", "rev": 4 }));
    stamp(&mut agent, "dashboard", 1_792_756_800_000);
    assert_eq!(agent["rev"], json!(5));
    assert_eq!(agent["updatedBy"], json!("dashboard"));
    assert_eq!(agent["updatedAt"], json!("2026-10-23T12:00:00.000Z"));
}

#[test]
fn the_claude_id_is_claude_id_else_the_older_hook_id_and_live_means_a_live_status() {
    assert_eq!(
        claude_id_of(&record(&json!({ "claudeId": "a", "hookId": "b" }))).as_deref(),
        Some("a")
    );
    assert_eq!(
        claude_id_of(&record(&json!({ "claudeId": "", "hookId": "b" }))).as_deref(),
        Some("b")
    );
    assert_eq!(claude_id_of(&record(&json!({}))), None);
    assert!(is_live(&record(&json!({ "status": "stop-requested" }))));
    assert!(!is_live(&record(
        &json!({ "status": "working", "archivedAt": "2026-10-23T12:00:00.000Z" })
    )));
    assert!(!is_live(&record(&json!({ "status": "done" }))));
}

#[test]
fn config_keeps_only_positive_numbers_and_the_options_follow_it() {
    let config = AgentsConfig::from_json(
        &json!({ "stallMs": 30000, "reapMs": 0, "idleWakeMs": "soon", "claudeHome": "C:/fake" }),
    );
    assert_eq!(config.stall_ms, Some(30_000));
    assert_eq!(config.reap_ms, None);
    assert_eq!(config.idle_wake_ms, None);
    let opts = ReconcilerOptions::default().configured(&config);
    assert_eq!(opts.stall_ms, 30_000);
    assert_eq!(opts.idle_wake_ms, 5 * 60_000);
    assert_eq!(opts.claude_home.to_string_lossy(), "C:/fake");
}

#[test]
fn the_file_system_lists_by_name_and_reads_a_range() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("b.txt"), "0123456789").unwrap();
    fs::write(dir.path().join("a.txt"), "x").unwrap();
    fs::create_dir(dir.path().join("c")).unwrap();
    let names: Vec<(String, bool)> = StdFs
        .list(dir.path())
        .unwrap()
        .into_iter()
        .map(|e| (e.name, e.is_dir))
        .collect();
    assert_eq!(
        names,
        vec![
            ("a.txt".to_owned(), false),
            ("b.txt".to_owned(), false),
            ("c".to_owned(), true)
        ]
    );
    assert_eq!(
        StdFs.read_at(&dir.path().join("b.txt"), 3, 4).unwrap(),
        b"3456"
    );
    assert_eq!(StdFs.stat(&dir.path().join("b.txt")).unwrap().size, 10);
    assert!(StdFs.stat(&dir.path().join("none")).is_none());
}

fn board(threads: &[Value]) -> Board {
    let file = board_json(threads, &[]);
    let list = file["threads"]
        .as_object()
        .unwrap()
        .values()
        .map(record)
        .collect();
    Board {
        agents: vec![],
        threads: list,
    }
}

#[test]
fn the_thread_to_wake_is_the_owner_while_it_lives_else_the_coordinator() {
    let board = board(&[
        thread_record("lead", "coordinator", &json!({})),
        thread_record("ui", "thread", &json!({})),
        thread_record("old", "thread", &json!({ "status": "done" })),
        thread_record("hold", "thread", &json!({ "placeholder": true })),
    ]);
    let wake = |thread: &str| board.wake_target(&record(&json!({ "thread": thread })));
    assert_eq!(wake("ui").as_deref(), Some("ui"));
    assert_eq!(wake("old").as_deref(), Some("lead"));
    assert_eq!(wake("hold").as_deref(), Some("lead"));
    assert_eq!(wake("nobody").as_deref(), Some("lead"));
    let ended_lead = Board {
        agents: vec![],
        threads: vec![record(&thread_record(
            "lead",
            "coordinator",
            &json!({ "status": "archived" }),
        ))],
    };
    assert_eq!(ended_lead.coordinator(), None);
}
