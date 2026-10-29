//! Ports `hook.guard.test.mjs`: `coord hook guard` (`PreToolUse` on the built-in browser's
//! `tabs_close`) denies it for a registered thread and stays silent for anyone else; the
//! session-start context tells a thread to keep its board open in the browser pane, asks once
//! for the board shortcut, and gives the wake line (Monitor wake-stream, a lead on `'*'`). The
//! wiring cases (`shared/hooks.json`, the render) stay with `ai-config`.

mod common;
mod hook_support;

use std::fs;
use std::path::{Path, PathBuf};

use common::Home;
use hook_support::{Fixture, LEAD, UI, hook_in, wire};
use serde_json::{Value, json};

const PORT: u16 = 51234;

/// The fixture with its checkout as its own git root and the dashboard on [`PORT`].
fn fixture() -> Fixture {
    let f = Fixture::new();
    fs::create_dir_all(f.cwd.join(".git")).unwrap();
    let config = f.home.dashboard_home().join("config.json");
    fs::write(config, format!(r#"{{"port":{PORT}}}"#)).unwrap();
    f
}

fn input(f: &Fixture, sid: &str) -> Value {
    json!({ "session_id": sid, "cwd": f.cwd_text(), "tool_name": "mcp__Claude_Browser__tabs_close" })
}

fn reason(out: &str) -> String {
    let json: Value = serde_json::from_str(out).expect("json out");
    assert_eq!(json["hookSpecificOutput"]["hookEventName"], "PreToolUse");
    assert_eq!(json["hookSpecificOutput"]["permissionDecision"], "deny");
    json["hookSpecificOutput"]["permissionDecisionReason"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn context(f: &Fixture, event: &str, input: &Value) -> String {
    let ran = f.hook(event, input);
    assert_eq!(ran.code, 0, "{}", ran.err);
    if event == "prompt" {
        return ran.out;
    }
    let out: Value = serde_json::from_str(&ran.out).expect("json out");
    out["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn a_coordinator_is_denied_with_its_board_url_on_the_configured_port() {
    let f = fixture();
    let ran = f.hook("guard", &input(&f, LEAD));
    assert_eq!(ran.code, 0);
    let text = reason(&ran.out);
    assert!(
        text.contains(&format!(
            "http://127.0.0.1:{PORT}/p/alpha/?thread=coordinator"
        )),
        "{text}"
    );
    assert_eq!(
        text,
        format!(
            "Do not close browser tabs in a registered thread: your board tab (http://127.0.0.1:{PORT}/p/alpha/?thread=coordinator) must stay open, the dashboard reports a closed board to the Overlord. Open other pages in a new tab (tabs_create), and if the board tab is gone, reopen it with preview_start."
        )
    );
}

#[test]
fn the_reason_falls_back_to_the_default_port_without_a_config() {
    let f = fixture();
    let bare = f.home.folder("no-config");
    let coord = f.home.coord_home().to_string_lossy().into_owned();
    let env = argus_coord::Env::of(&[
        ("COORD_HOME", &coord),
        ("DASHBOARD_HOME", &bare.to_string_lossy()),
    ]);
    let ran = hook_support::hook_with(env, "guard", &input(&f, LEAD).to_string(), &f.cwd);
    assert!(reason(&ran.out).contains("http://127.0.0.1:47615/p/alpha/?thread=coordinator"));
}

#[test]
fn the_port_of_the_running_server_wins_over_the_config() {
    let f = fixture();
    let pid = f.home.dashboard_home().join("server.pid");
    fs::write(pid, r#"{"pid":1,"port":40001}"#).unwrap();
    let ran = f.hook("guard", &input(&f, UI));
    assert!(reason(&ran.out).contains("http://127.0.0.1:40001/p/alpha/?thread=ui"));
}

/// The guard reason of `sid` when `server.pid` holds `record`; no server is running, and none is
/// asked.
fn guard_reason_with_pid(record: &Value, sid: &str) -> String {
    let f = fixture();
    let pid = f.home.dashboard_home().join("server.pid");
    fs::write(pid, record.to_string()).unwrap();
    let ran = f.hook("guard", &input(&f, sid));
    reason(&ran.out)
}

#[test]
fn with_the_localhost_door_in_the_pid_file_the_board_url_uses_the_localhost_name() {
    let text = guard_reason_with_pid(&json!({ "pid": 1, "port": 40001, "localhostDoor": 80 }), UI);
    assert!(
        text.contains("(http://alpha.localhost/?thread=ui)"),
        "{text}"
    );
}

#[test]
fn without_the_localhost_door_in_the_pid_file_the_board_url_stays_on_the_loopback_port() {
    let text = guard_reason_with_pid(&json!({ "pid": 1, "port": 40001 }), UI);
    assert!(
        text.contains("(http://127.0.0.1:40001/p/alpha/?thread=ui)"),
        "{text}"
    );
}

#[test]
fn a_registered_thread_is_denied_with_its_own_board_url() {
    let f = fixture();
    let ran = f.hook("guard", &input(&f, UI));
    assert!(reason(&ran.out).contains(&format!("http://127.0.0.1:{PORT}/p/alpha/?thread=ui")));
}

#[test]
fn an_unknown_session_a_placeholder_and_a_project_with_no_board_get_nothing() {
    let f = fixture();
    let stranger = f.hook("guard", &input(&f, "sid-stranger-0003"));
    assert_eq!(
        (stranger.code, stranger.out.as_str(), stranger.err.as_str()),
        (0, "", "")
    );
    f.hook(
        "session-start",
        &json!({ "session_id": "sid-new-0009", "cwd": f.cwd_text() }),
    );
    let placeholder = f.hook("guard", &input(&f, "sid-new-0009"));
    assert_eq!((placeholder.code, placeholder.out.as_str()), (0, ""));
    let ghost = f.home.folder("checkout").join("ghost");
    wire(&ghost, "ghost");
    let body = json!({ "session_id": "sid-stranger-0003", "cwd": ghost });
    let ran = hook_in(&f.home, "guard", &body, &ghost);
    assert_eq!((ran.code, ran.out.as_str()), (0, ""));
}

#[test]
fn the_guard_never_changes_the_board() {
    let f = fixture();
    let before = f.raw();
    f.hook("guard", &input(&f, LEAD));
    f.hook("guard", &input(&f, "sid-stranger-0003"));
    assert_eq!(f.raw(), before);
}

#[test]
fn the_coordinator_is_told_to_open_its_board_in_the_browser_pane_and_keep_it_open() {
    let f = fixture();
    let text = context(&f, "session-start", &input(&f, LEAD));
    for part in [
        "preview_start",
        "Never close that tab",
        &format!("http://127.0.0.1:{PORT}/p/alpha/?thread=coordinator"),
        "30 seconds",
        "mcp__ccd_view__show_pane with pane \"preview\"",
    ] {
        assert!(text.contains(part), "{part}: {text}");
    }
    assert!(text.starts_with("You are thread \"coordinator\" on the Alpha coordination board (subject lead). Load the coord skill before working; keep your note current with coord thread note.\nDashboard: "));
}

#[test]
fn a_thread_gets_the_same_line_with_its_own_id() {
    let f = fixture();
    let text = context(&f, "session-start", &input(&f, UI));
    assert!(text.contains("You are thread \"ui\""), "{text}");
    assert!(text.contains(&format!("http://127.0.0.1:{PORT}/p/alpha/?thread=ui")));
    assert!(text.contains("Never close that tab"));
}

fn launch(root: &Path, configs: &Value) -> PathBuf {
    let file = root.join(".claude").join("launch.json");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(
        &file,
        json!({ "version": "0.0.1", "configurations": configs }).to_string(),
    )
    .unwrap();
    file
}

#[test]
fn a_checkout_with_no_shortcut_to_the_board_is_told_to_run_board_shortcut_once() {
    let f = fixture();
    let say = "run `coord board shortcut` once (idempotent)";
    for sid in [LEAD, UI] {
        assert!(
            context(&f, "session-start", &input(&f, sid)).contains(say),
            "{sid}"
        );
    }
    let other = json!([{ "name": "Dev", "port": 3000 }, { "name": "x", "port": PORT, "url": format!("http://other.alpha.localhost:{PORT}") }]);
    launch(&f.cwd, &other);
    assert!(context(&f, "session-start", &input(&f, UI)).contains(say));
    let both = json!([{ "name": "Alpha board", "port": 1, "url": "http://alpha.localhost:1" }, { "name": "Alpha board: ui", "port": PORT, "url": format!("http://ui.alpha.localhost:{PORT}") }]);
    let file = launch(&f.cwd, &both);
    for sid in [LEAD, UI] {
        let text = context(&f, "session-start", &input(&f, sid));
        assert!(!text.contains("board shortcut"), "{text}");
        assert!(text.contains("preview_start"));
    }
    fs::write(file, "{ not json").unwrap();
    assert!(!context(&f, "session-start", &input(&f, UI)).contains("board shortcut"));
}

fn overlord_home(home: &Home, root: &Path) {
    wire(root, "portfolio");
    fs::create_dir_all(root.join(".git")).unwrap();
    let thread = json!({
        "id": "coordinator", "name": "coordinator", "role": "coordinator", "status": "active",
        "subject": null, "root": root, "session": "Overlord", "sessionId": null,
        "rev": 1, "rank": 1024, "lastSeen": hook_support::ago(1), "activity": "working",
    });
    let board = json!({
        "schema": 2, "project": "portfolio", "name": "Portfolio", "counters": {},
        "threads": { "coordinator": thread }, "inbox": {}, "agents": {}, "dirty": {},
    });
    home.write_board("portfolio", &board);
}

#[test]
fn the_overlord_is_told_the_shortcut_makes_two_rows_until_both_are_there() {
    let f = fixture();
    let root = f.home.folder("checkout").join("portfolio");
    overlord_home(&f.home, &root);
    let body = json!({ "session_id": "sid-overlord-0007", "cwd": root });
    let start = || {
        let ran = hook_in(&f.home, "session-start", &body, &root);
        let out: Value = serde_json::from_str(&ran.out).expect("json");
        out["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let text = start();
    assert!(text.contains("creates two rows, \"Argus\" (the overview of every project) and \"Overlord\" (your own board)"), "{text}");
    assert!(text.contains("is reported to the Overlord, who resolves it with you at once"));
    assert!(
        text.contains("--id \"Overlord\""),
        "labelled with the session title"
    );
    assert!(text.contains("on the Overlord coordination board"));
    launch(
        &root,
        &json!([{ "name": "Argus", "port": 1, "url": "http://argus.localhost:1" }]),
    );
    assert!(start().contains("creates two rows"));
    let both = json!([{ "name": "Argus", "port": 1, "url": "http://argus.localhost:1" }, { "name": "Overlord", "port": 2, "url": "http://overlord.localhost:2" }]);
    launch(&root, &both);
    assert!(!start().contains("board shortcut"));
    let lead = context(&f, "session-start", &input(&f, LEAD));
    assert!(!lead.contains("creates two rows"));
}

#[test]
fn only_session_start_asks_for_the_shortcut_and_gives_the_wake_line() {
    let f = fixture();
    let mut board = f.board();
    board["inbox"]["ui"] = json!([{ "id": "m-1", "from": "coordinator", "kind": "note", "text": "hi", "deliveredAt": null }]);
    f.save(&board);
    let out = context(&f, "prompt", &input(&f, UI));
    assert!(out.starts_with("You are thread \"ui\""), "{out}");
    assert!(!out.contains("board shortcut"), "{out}");
    assert!(!out.contains("wake-stream"), "{out}");
}

#[test]
fn the_lead_is_told_to_run_its_wake_stream_for_every_thread_under_monitor() {
    let f = fixture();
    let text = context(&f, "session-start", &input(&f, LEAD));
    assert!(text.contains("Waking: start your wake-stream now with the Monitor tool (persistent true if it offers it, else its longest timeout): `dashboard wake-stream --project alpha --thread '*' --id \"Alpha\"`"), "{text}");
    assert!(text.contains(
        "Do not use a coord tick cron; if you have one, delete it (CronList, then CronDelete)"
    ));
}

#[test]
fn a_thread_listens_to_its_own_id() {
    let f = fixture();
    let text = context(&f, "session-start", &input(&f, UI));
    assert!(
        text.contains("`dashboard wake-stream --project alpha --thread ui --id \"Alpha - UI\"`"),
        "{text}"
    );
    assert!(!text.contains("--thread '*'"));
}

#[test]
fn an_adopted_session_gets_the_join_text_then_the_reminder_and_no_wake_line() {
    let f = fixture();
    let body = json!({ "session_id": "sid-brand-new-0004", "cwd": f.cwd_text() });
    let text = context(&f, "session-start", &body);
    assert_eq!(
        text,
        "Coordination board: this session is part of Alpha, run by its lead session \"Alpha\". It is on the board as a placeholder thread \"s-sidbra\". Now: load the coord skill; once you know what this session works on, rename it \"Alpha - <Subject>\" (set_session_title on \"self\", subject four words at most) and run coord thread register <short-name> --subject <subject> --session \"Alpha - <Subject>\". If the work belongs to another project, register there with --project <key> instead of alpha."
    );
    let again = context(&f, "session-start", &body);
    assert_eq!(
        again,
        "Coordination board: this session is still the unnamed placeholder \"s-sidbra\". Name and register it (coord skill, SKILL.md step 1)."
    );
}
