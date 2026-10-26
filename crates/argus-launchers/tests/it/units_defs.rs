//! Pure rules: definitions read from a board, the final verdict of a run, timestamps, the
//! agents join, image sniffing and the shape of the page doc.

use std::sync::Arc;

use argus_launchers::{
    AgentFilter, BoardFileDefs, DefsSource, EndInput, ExitInfo, ListFilter, MemoryDefs, RunDef,
    RunKind, RunStatus, agents_with_runs, check_agent, check_key, check_name, check_rid,
    coerce_def, defs_from_map, format_iso, parse_iso, public_def, sniff_image, verdict,
};
use serde_json::{Value, json};

#[test]
fn build_and_test_definitions_default_to_1800_seconds_and_zero_means_none() {
    let build = coerce_def("build", &json!({"cmd": "x", "kind": "build"})).unwrap();
    let free = coerce_def(
        "free",
        &json!({"cmd": "x", "kind": "build", "timeoutSec": 0}),
    )
    .unwrap();
    let test = coerce_def("t", &json!({"cmd": "x", "kind": "test"})).unwrap();
    let task = coerce_def("task", &json!({"cmd": "x"})).unwrap();
    assert_eq!(
        (
            build.timeout_sec,
            free.timeout_sec,
            test.timeout_sec,
            task.timeout_sec
        ),
        (1800, 0, 1800, 0)
    );
    let odd = coerce_def(
        "odd",
        &json!({"cmd": "x", "timeoutSec": 999_999, "parallel": 99, "rev": "7"}),
    )
    .unwrap();
    assert_eq!(
        (odd.timeout_sec, odd.parallel, odd.rev),
        (0, 1, 7),
        "out of range falls back"
    );
}

#[test]
fn a_definition_without_a_command_or_with_junk_is_dropped_and_the_rest_filled_in() {
    assert!(coerce_def("x", &json!({"cmd": "   "})).is_none());
    assert!(coerce_def("x", &json!({"kind": "build"})).is_none());
    assert!(coerce_def("x", &json!("echo")).is_none());
    let def = coerce_def(
        "x",
        &json!({"cmd": "run", "kind": "weird", "env": {"A": "1", "B": 2}, "bundle": ""}),
    )
    .unwrap();
    assert_eq!(def.kind, RunKind::Task);
    assert_eq!(def.env.len(), 1, "non-string env values are dropped");
    assert_eq!(
        (def.bundle, def.defined_by.as_str(), def.cwd.as_str()),
        (None, "unknown", "")
    );
}

#[test]
fn a_launcher_or_bundle_definition_is_visible_only_with_window_a_plain_one_with_visible() {
    let visible = |raw: Value| coerce_def("d", &raw).unwrap().visible;
    assert!(!visible(
        json!({"cmd": "x", "hand": true, "launcher": "m", "visible": true})
    ));
    assert!(visible(
        json!({"cmd": "x", "hand": true, "launcher": "m", "window": true})
    ));
    assert!(!visible(
        json!({"cmd": "x", "hand": true, "bundle": "b", "visible": true})
    ));
    assert!(visible(json!({"cmd": "x", "hand": true, "visible": true})));
    assert!(
        !visible(json!({"cmd": "x", "visible": true})),
        "visible needs hand"
    );
}

#[test]
fn the_names_of_a_map_are_checked_sorted_and_the_public_copy_has_no_env() {
    let map = json!({"zed": {"cmd": "z"}, "alpha": {"cmd": "a", "env": {"K": "v"}}, "Bad Name": {"cmd": "b"}});
    let defs = defs_from_map(map.as_object().unwrap());
    assert_eq!(
        defs.iter().map(|d| d.name.as_str()).collect::<Vec<_>>(),
        ["alpha", "zed"]
    );
    assert!(public_def(&defs[0]).env.is_empty());
    assert_eq!(defs[0].env.len(), 1);
}

#[test]
fn the_label_the_board_keeps_is_not_part_of_the_public_definition() {
    let raw = json!({"cmd": "x", "label": "Play alpha"});
    let def = coerce_def("launcher-alpha", &raw).unwrap();
    assert_eq!(
        def.label.as_deref(),
        Some("Play alpha"),
        "the board keeps it"
    );
    let public = serde_json::to_value(public_def(&def)).unwrap();
    assert!(public.get("label").is_none(), "{public}");
    let bare = serde_json::to_value(RunDef::default()).unwrap();
    assert!(bare.get("label").is_none(), "{bare}");
}

#[test]
fn the_board_file_is_read_on_every_call_and_a_missing_file_falls_back() {
    let dir = tempfile::TempDir::new().unwrap();
    let fallback = Arc::new(MemoryDefs::new());
    fallback.set(
        "alpha",
        vec![RunDef {
            name: "kept".to_owned(),
            cmd: "k".to_owned(),
            ..RunDef::default()
        }],
    );
    let source = BoardFileDefs::new(dir.path(), Some(fallback as Arc<dyn DefsSource>));
    assert_eq!(
        source.defs("alpha")[0].name,
        "kept",
        "no file: the last good copy"
    );
    std::fs::create_dir_all(dir.path().join("alpha")).unwrap();
    let board = json!({"project": "alpha", "runDefs": {"fresh": {"cmd": "f", "launcher": "fresh", "hand": true}}});
    std::fs::write(
        dir.path().join("alpha").join("board.json"),
        board.to_string(),
    )
    .unwrap();
    let defs = source.defs("alpha");
    assert_eq!(
        (
            defs.len(),
            defs[0].name.as_str(),
            defs[0].launcher.as_deref()
        ),
        (1, "fresh", Some("fresh"))
    );
    std::fs::write(dir.path().join("alpha").join("board.json"), "{ half").unwrap();
    assert_eq!(
        source.defs("alpha")[0].name,
        "kept",
        "a half-written file falls back too"
    );
    assert!(source.defs("nobody").is_empty());
}

#[test]
fn names_keys_run_ids_and_agent_ids_have_their_shapes() {
    assert!(check_key("alpha-2").is_ok() && check_key("Alpha").is_err() && check_key("").is_err());
    assert!(
        check_name("a".repeat(40).as_str()).is_ok() && check_name("a".repeat(41).as_str()).is_err()
    );
    assert!(check_rid("R-12").is_ok() && check_rid("R-").is_err() && check_rid("r-1").is_err());
    assert!(check_agent("A-3").is_ok() && check_agent("x").is_err());
    assert_eq!(
        check_key("Bad Key").unwrap_err().message,
        "\"Bad Key\" is not a project key"
    );
}

fn end(
    stop_by: Option<&str>,
    timed_out: bool,
    exit: Option<(Option<i32>, Option<&str>)>,
) -> argus_launchers::Verdict {
    verdict(&EndInput {
        stop_by: stop_by.map(str::to_owned),
        timed_out,
        timeout_sec: 30,
        exit: exit.map(|(code, signal)| ExitInfo {
            code,
            signal: signal.map(str::to_owned),
        }),
    })
}

#[test]
fn the_verdict_a_stop_wins_over_a_timeout_which_wins_over_the_exit() {
    let stopped = end(Some("me"), true, Some((Some(0), None)));
    assert_eq!(
        (stopped.status, stopped.error, stopped.note.as_str()),
        (RunStatus::Stopped, None, "stopped by me")
    );
    let timed = end(None, true, Some((Some(1), None)));
    assert_eq!(
        (timed.status, timed.error.as_deref()),
        (RunStatus::Failed, Some("timeout"))
    );
    assert_eq!(timed.note, "timed out after 30s and was killed");
    let ok = end(None, false, Some((Some(0), None)));
    assert_eq!(
        (ok.status, ok.note.as_str()),
        (RunStatus::Exited, "exited with code 0")
    );
    let failed = end(None, false, Some((Some(3), None)));
    assert_eq!(
        (failed.status, failed.note.as_str()),
        (RunStatus::Failed, "exited with code 3")
    );
    let signalled = end(None, false, Some((None, Some("SIGKILL"))));
    assert_eq!(signalled.note, "ended by signal SIGKILL");
    assert_eq!(end(None, false, None).status, RunStatus::Failed);
}

#[test]
fn timestamps_round_trip_and_parse_the_forms_a_board_holds() {
    let ms = 1_793_000_000_123;
    assert_eq!(parse_iso(&format_iso(ms)), Some(ms));
    assert_eq!(format_iso(0), "1970-01-01T00:00:00.000Z");
    assert_eq!(
        parse_iso("2026-10-24T00:00:00.000Z"),
        Some(1_792_800_000_000)
    );
    assert_eq!(parse_iso("2026-10-24T00:00:00Z"), Some(1_792_800_000_000));
    assert_eq!(parse_iso("2026-10-24T00:00:00.5Z"), Some(1_792_800_000_500));
    assert_eq!(
        parse_iso("2024-02-29T12:00:00.000Z")
            .map(format_iso)
            .as_deref(),
        Some("2024-02-29T12:00:00.000Z")
    );
    assert_eq!(
        (parse_iso(""), parse_iso("yesterday"), parse_iso("2026-1-1")),
        (None, None, None)
    );
}

#[test]
fn agents_join_the_runs_that_carry_their_id_newest_first_with_the_live_and_thread_filters() {
    let agents = vec![
        json!({"id": "A-1", "name": "builder", "thread": "T1", "status": "working"}),
        json!({"id": "A-2", "name": "old", "thread": "T2", "status": "done"}),
        json!({"id": "A-3", "name": "gone", "thread": "T1", "status": "working", "deletedAt": "2026-01-01"}),
    ];
    let runs = vec![
        json!({"id": "R-1", "agent": "A-1", "reason": "first", "queuedAt": "2026-01-01T00:00:00.000Z"}),
        json!({"id": "R-2", "agent": "A-1", "reason": "second", "queuedAt": "2026-01-02T00:00:00.000Z"}),
        json!({"id": "R-3", "agent": null, "queuedAt": "2026-01-03T00:00:00.000Z"}),
    ];
    let all = agents_with_runs(&agents, &runs, &AgentFilter::default());
    assert_eq!(all.len(), 3);
    let ids = |agent: &Value| {
        agent["runs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    };
    assert_eq!(ids(&all[0]), ["R-2", "R-1"]);
    assert!(ids(&all[1]).is_empty());
    let live = agents_with_runs(
        &agents,
        &runs,
        &AgentFilter {
            live: true,
            thread: None,
        },
    );
    assert_eq!(live.len(), 1);
    let mine = agents_with_runs(
        &agents,
        &runs,
        &AgentFilter {
            live: false,
            thread: Some("T2".to_owned()),
        },
    );
    assert_eq!(mine[0]["id"], "A-2");
}

#[test]
fn images_are_known_by_their_magic_number() {
    assert_eq!(
        sniff_image(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 1]),
        Some(("image/png", ".png"))
    );
    assert_eq!(
        sniff_image(&[0xff, 0xd8, 0xff, 0xe0]),
        Some(("image/jpeg", ".jpg"))
    );
    assert_eq!(sniff_image(b"GIF89a.."), Some(("image/gif", ".gif")));
    assert_eq!(
        sniff_image(b"RIFF\0\0\0\0WEBPVP8 "),
        Some(("image/webp", ".webp"))
    );
    assert_eq!(sniff_image(b"<html>"), None);
    assert_eq!(sniff_image(b""), None);
}

#[tokio::test(start_paused = true)]
async fn the_page_doc_hides_the_command_the_folder_and_the_log_cursor_and_filters_parse() {
    let env = crate::common::Env::new();
    env.define("alpha", vec![env.def("hello", "echo secret-looking")]);
    let row = env.start("alpha", "hello");
    let doc = row.to_doc();
    for hidden in ["scope", "cmd", "cwd", "logSeq"] {
        assert!(
            doc.get(hidden).is_none(),
            "{hidden} must not reach the page"
        );
    }
    assert_eq!(
        (doc["id"].as_str(), doc["status"].as_str()),
        (Some("R-1"), Some("running"))
    );
    let filter = ListFilter::default().with_statuses("running, queued,,");
    assert_eq!(filter.statuses, ["running", "queued"]);
    assert_eq!(
        ListFilter {
            limit: Some(9999),
            ..ListFilter::default()
        }
        .row_limit(),
        500
    );
    assert_eq!(
        ListFilter {
            limit: Some(-3),
            ..ListFilter::default()
        }
        .row_limit(),
        50
    );
}
