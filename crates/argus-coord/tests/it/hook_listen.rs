//! Ports `hook.listen.test.mjs`: `coord hook listen`, the async and `asyncRewake` hook that is a
//! session's wake listener. A small stand-in for the dashboard's wake stream serves the events
//! (the replay of a queued wake and the replacement of an older listener are the server's, in
//! `argus-wake`): a wake ends the listener with exit 2 and its line on stderr, `replaced` ends
//! it silently, a lead listens to `*`, an unregistered session and a wait that ran out exit 0
//! with nothing to say, and a `PostToolUse` listener skips while a fresh one of the thread lives.
//! The parent watch (`ARGUS_COORD_PARENT`) runs the built binary and skips without it.

use std::env;
use std::fs;
use std::io::Write;
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::hook_support::{Fixture, LEAD, StandIn, UI, fixture, hook_with};
use argus_coord::Env;
use serde_json::{Value, json};

/// What the stand-in answers on a stream: the events to send, then it holds the stream open
/// (`None`: hold it open, silent; an empty text: end the stream at once).
type Answer = Arc<dyn Fn(usize) -> Option<String> + Send + Sync>;

fn answer(mut stream: TcpStream, events: Option<String>) {
    let head = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n";
    stream.write_all(head.as_bytes()).ok();
    stream.write_all(b": hello\n\n").ok();
    if let Some(events) = events {
        if events.is_empty() {
            return;
        }
        stream.write_all(events.as_bytes()).ok();
    }
    stream.flush().ok();
    thread::sleep(Duration::from_secs(20));
}

fn serve(answer_with: Answer) -> StandIn {
    StandIn::start(move |n, stream, _| {
        let events = answer_with(n);
        thread::spawn(move || answer(stream, events));
    })
}

fn wake(data: &Value) -> String {
    format!("event: wake\ndata: {data}\n\n")
}

fn always(events: Option<String>) -> Answer {
    Arc::new(move |_| events.clone())
}

fn env_of(f: &Fixture, extra: &[(&str, &str)]) -> Env {
    let coord = f.home.coord_home().to_string_lossy().into_owned();
    let dash = f.home.dashboard_home().to_string_lossy().into_owned();
    let mut pairs = vec![
        ("COORD_HOME", coord.as_str()),
        ("DASHBOARD_HOME", dash.as_str()),
        ("COORD_LISTEN_MS", "15000"),
    ];
    pairs.extend_from_slice(extra);
    Env::of(&pairs)
}

fn listen(f: &Fixture, sid: &str, event: &str, extra: &[(&str, &str)]) -> crate::common::Ran {
    let input = json!({ "session_id": sid, "cwd": f.cwd_text(), "hook_event_name": event });
    hook_with(env_of(f, extra), "listen", &input.to_string(), &f.cwd)
}

#[test]
fn a_wake_for_the_thread_ends_the_listener_with_exit_2_and_the_line_on_stderr() {
    let server = serve(always(Some(wake(
        &json!({ "thread": "ui", "lines": ["lead: look at the build"], "count": 1 }),
    ))));
    let f = fixture(&server);
    let ran = listen(&f, UI, "Stop", &[]);
    assert_eq!(ran.code, 2);
    assert_eq!(
        ran.err,
        "WAKE ui: lead: look at the build. Run: coord inbox\n"
    );
    assert_eq!(ran.out, "");
    assert_eq!(
        server.paths(),
        ["/api/p/alpha/wakes/stream?thread=ui&v=2&id=Alpha%20-%20UI"]
    );
}

#[test]
fn several_wakes_are_counted_and_unreadable_data_still_wakes() {
    let data = json!({ "thread": "ui", "lines": ["a", "b"], "count": 3 });
    let server = serve(always(Some(wake(&data))));
    let f = fixture(&server);
    assert_eq!(
        listen(&f, UI, "Stop", &[]).err,
        "WAKE ui: a; b (3 wakes). Run: coord inbox\n"
    );
    let bad = serve(always(Some("event: wake\ndata: not json\n\n".to_owned())));
    let f = fixture(&bad);
    let ran = listen(&f, UI, "Stop", &[]);
    assert_eq!(
        (ran.code, ran.err.as_str()),
        (2, "WAKE ui: see coord inbox\n")
    );
}

#[test]
fn a_newer_listener_replaces_this_one_which_ends_silently_with_exit_0() {
    let server = serve(always(Some("event: replaced\ndata: {}\n\n".to_owned())));
    let f = fixture(&server);
    let ran = listen(&f, UI, "Stop", &[]);
    assert_eq!((ran.code, ran.out.as_str(), ran.err.as_str()), (0, "", ""));
}

#[test]
fn a_lead_listens_for_every_thread_of_its_project() {
    let server = serve(always(Some(wake(
        &json!({ "thread": "ui", "lines": ["ui needs something"] }),
    ))));
    let f = fixture(&server);
    let ran = listen(&f, LEAD, "SessionStart", &[]);
    assert_eq!(ran.code, 2);
    assert_eq!(ran.err, "WAKE ui: ui needs something. Run: coord inbox\n");
    assert_eq!(
        server.paths(),
        ["/api/p/alpha/wakes/stream?thread=*&v=2&id=Alpha"]
    );
}

#[test]
fn an_unregistered_session_and_a_wait_that_ran_out_exit_0_with_nothing_to_say() {
    let server = serve(always(None));
    let f = fixture(&server);
    let stranger = listen(&f, "sid-stranger", "Stop", &[]);
    assert_eq!((stranger.code, stranger.err.as_str()), (0, ""));
    assert!(server.paths().is_empty(), "no stream for a stranger");
    let began = Instant::now();
    let patient = listen(&f, UI, "Stop", &[("COORD_LISTEN_MS", "700")]);
    assert_eq!((patient.code, patient.err.as_str()), (0, ""));
    assert!(
        began.elapsed() < Duration::from_secs(4),
        "{:?}",
        began.elapsed()
    );
}

#[test]
fn a_stream_that_ended_is_joined_again_and_the_wake_still_arrives() {
    let answer_with: Answer = Arc::new(|n| {
        Some(if n == 0 {
            String::new()
        } else {
            wake(&json!({ "thread": "ui", "lines": ["second try"] }))
        })
    });
    let server = serve(answer_with);
    let f = fixture(&server);
    let ran = listen(&f, UI, "Stop", &[("COORD_LISTEN_MS", "20000")]);
    assert_eq!(ran.code, 2);
    assert_eq!(ran.err, "WAKE ui: second try. Run: coord inbox\n");
    assert_eq!(server.paths().len(), 2);
}

fn stamp(f: &Fixture) -> PathBuf {
    f.home.dashboard_home().join("listen-alpha-ui.json")
}

/// A process that lives for a while and is not this one.
fn other_process() -> Child {
    let mut command = if cfg!(windows) {
        let mut c = Command::new("ping");
        c.args(["-n", "30", "127.0.0.1"]);
        c
    } else {
        let mut c = Command::new("sleep");
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
    assert_eq!(server.paths().len(), 1, "no second stream");
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

#[test]
fn session_start_says_wakes_are_automatic_only_where_the_settings_carry_the_listen_hook() {
    let server = serve(always(None));
    let f = fixture(&server);
    let start = || {
        let ran = f.hook(
            "session-start",
            &json!({ "session_id": UI, "cwd": f.cwd_text() }),
        );
        let out: Value = serde_json::from_str(&ran.out).unwrap();
        out["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let before = start();
    assert!(before.contains("start your wake-stream now with the Monitor tool"));
    assert!(!before.contains("Waking: automatic"));
    let settings = json!({ "hooks": { "Stop": [{ "hooks": [{ "type": "command", "command": "node .claude/tools/coord/coord.mjs hook listen", "async": true, "asyncRewake": true }] }] } });
    fs::create_dir_all(f.cwd.join(".claude")).unwrap();
    fs::write(
        f.cwd.join(".claude").join("settings.json"),
        settings.to_string(),
    )
    .unwrap();
    let after = start();
    assert!(after.contains("Waking: automatic. A background hook is your wake listener"));
    assert!(after.contains("Do NOT start a Monitor wake-stream"));
    assert!(!after.contains("with the Monitor tool"));
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
fn a_listener_ends_silently_when_its_parent_is_gone() {
    let Some(argus) = argus_exe() else {
        eprintln!("skipped: no argus binary (cargo build -p argus)");
        return;
    };
    let server = serve(always(None));
    let f = fixture(&server);
    let mut parent = other_process();
    let parent_pid = parent.id().to_string();
    let mut child = Command::new(argus);
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
