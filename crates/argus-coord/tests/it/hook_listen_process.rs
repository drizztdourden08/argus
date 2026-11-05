//! E2: the listener hook with real processes: a stamp of a live other process, and a listener of
//! the built `argus` binary that ends when its parent is gone.

// test-tag: e2e

use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use argus_core::process::hidden_command;

use crate::hook_support::listen::{always, env_of, listen, serve, streams, wake};
use crate::hook_support::{Fixture, UI, fixture};
use serde_json::{Value, json};

fn stamp(f: &Fixture) -> PathBuf {
    f.home.dashboard_home().join("listen-alpha-ui.json")
}

/// A process that lives for a while and is not this one.
fn other_process() -> Child {
    let mut command = if cfg!(windows) {
        let mut c = hidden_command("ping");
        c.args(["-n", "30", "127.0.0.1"]);
        c
    } else {
        let mut c = hidden_command("sleep");
        c.arg("30");
        c
    };
    command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}

#[test]
#[ignore = "e2e"]
fn a_tool_use_listener_skips_while_a_fresh_one_lives_and_a_stop_always_takes_over() {
    let server = serve(always(Some(wake(
        &json!({ "thread": "ui", "lines": ["go"] }),
    ))));
    let f = fixture(&server);
    assert_eq!(
        listen(&f, UI, "PostToolUse", &[]).code,
        2,
        "no stamp: it listens"
    );
    let written: Value = serde_json::from_str(&fs::read_to_string(stamp(&f)).unwrap()).unwrap();
    assert_eq!(written["pid"], std::process::id());
    let mut other = other_process();
    let fresh = json!({ "pid": other.id(), "at": argus_core::clock::now_ms() });
    fs::write(stamp(&f), fresh.to_string()).unwrap();
    let skipped = listen(&f, UI, "PostToolUse", &[]);
    assert_eq!((skipped.code, skipped.err.as_str()), (0, ""));
    assert_eq!(streams(&server).len(), 1, "no second stream");
    assert_eq!(listen(&f, UI, "Stop", &[]).code, 2, "a Stop takes over");
    other.kill().ok();
    other.wait().ok();
    let old = json!({ "pid": other.id(), "at": argus_core::clock::now_ms() });
    fs::write(stamp(&f), old.to_string()).unwrap();
    assert_eq!(
        listen(&f, UI, "PostToolUse", &[]).code,
        2,
        "a dead listener is replaced"
    );
}

fn read_stamp(f: &Fixture) -> Value {
    serde_json::from_str(&fs::read_to_string(stamp(f)).unwrap()).unwrap()
}

fn write_stamp(f: &Fixture, pid: u32, at: i64, started: i64) {
    let old = json!({ "pid": pid, "at": at, "started": started });
    fs::write(stamp(f), old.to_string()).unwrap();
}

#[test]
#[ignore = "e2e"]
fn the_tool_hook_keeps_a_live_old_listener_fresh_so_the_listener_after_it_has_nothing_to_do() {
    let server = serve(always(None));
    let f = fixture(&server);
    let mut other = other_process();
    let now = argus_core::clock::now_ms();
    let started = now - 10 * 60_000;
    write_stamp(&f, other.id(), started, started);
    let tool = json!({ "session_id": UI, "cwd": f.cwd_text() });
    assert_eq!(f.hook("tool", &tool).out, "");
    let kept = read_stamp(&f);
    assert_eq!(kept["pid"], other.id());
    assert_eq!(kept["started"], started);
    assert!(kept["at"].as_i64().unwrap() >= now, "{kept}");
    let skipped = listen(&f, UI, "PostToolUse", &[]);
    assert_eq!((skipped.code, skipped.err.as_str()), (0, ""));
    assert!(server.paths().is_empty(), "no stream was opened");
    other.kill().ok();
    other.wait().ok();
}

#[test]
#[ignore = "e2e"]
fn the_tool_hook_leaves_a_dead_a_spent_or_a_sub_agents_listener_stamp_as_it_is() {
    let server = serve(always(None));
    let f = fixture(&server);
    let tool = json!({ "session_id": UI, "cwd": f.cwd_text() });
    let now = argus_core::clock::now_ms();
    let old = now - 10 * 60_000;
    let mut other = other_process();
    write_stamp(&f, other.id(), old, now - 7 * 3_600_000);
    f.hook("tool", &tool);
    assert_eq!(read_stamp(&f)["at"], old, "past its own deadline");
    write_stamp(&f, other.id(), old, old);
    let mut sub = tool.clone();
    sub["agent_id"] = json!("a1b2c3d4e5f6a7b8c");
    f.hook("tool", &sub);
    assert_eq!(read_stamp(&f)["at"], old, "a sub-agent keeps nothing");
    other.kill().ok();
    other.wait().ok();
    write_stamp(&f, other.id(), old, old);
    f.hook("tool", &tool);
    assert_eq!(read_stamp(&f)["at"], old, "a dead listener is not kept");
    fs::remove_file(stamp(&f)).unwrap();
    f.hook("tool", &tool);
    assert!(!stamp(&f).exists(), "the tool hook never starts a listener");
}

fn argus_exe() -> Option<PathBuf> {
    if let Some(named) = env::var_os("ARGUS_EXE") {
        return Some(PathBuf::from(named)).filter(|p| p.is_file());
    }
    let exe = env::current_exe().ok()?;
    let beside = exe
        .parent()?
        .parent()?
        .join(format!("argus{}", env::consts::EXE_SUFFIX));
    beside.is_file().then_some(beside)
}

#[test]
#[ignore = "e2e"]
fn a_listener_ends_silently_when_its_parent_is_gone() {
    let Some(argus) = argus_exe() else {
        eprintln!("skipped: no argus binary (cargo build -p argus)");
        return;
    };
    let server = serve(always(None));
    let f = fixture(&server);
    let mut parent = other_process();
    let parent_pid = parent.id().to_string();
    let mut child = hidden_command(argus);
    for (k, v) in env_of(
        &f,
        &[
            ("COORD_LISTEN_MS", "60000"),
            ("ARGUS_COORD_PARENT", &parent_pid),
        ],
    )
    .iter()
    {
        child.env(k, v);
    }
    let input = json!({ "session_id": UI, "cwd": f.cwd_text(), "hook_event_name": "Stop" });
    let mut child = child
        .args(["coord", "hook", "listen"])
        .current_dir(&f.cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    parent.kill().ok();
    parent.wait().ok();
    let began = Instant::now();
    let out = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if began.elapsed() > Duration::from_secs(15) {
            child.kill().ok();
            panic!("the listener outlived its parent");
        }
        thread::sleep(Duration::from_millis(100));
    };
    assert_eq!(out.code(), Some(0));
    assert_eq!(server.paths().len(), 1, "it was listening");
}
