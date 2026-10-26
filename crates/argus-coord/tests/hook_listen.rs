//! Ports `hook.listen.test.mjs`: `coord hook listen`, the async and `asyncRewake` hook that is a
//! session's wake listener. A small stand-in for the dashboard's wake stream serves the events
//! (the replay of a queued wake and the replacement of an older listener are the server's, in
//! `argus-wake`): a wake ends the listener with exit 2 and its line on stderr, `replaced` ends
//! it silently, a lead listens to `*`, an unregistered session and a wait that ran out exit 0
//! with nothing to say, and a `PostToolUse` listener skips while a fresh one of the thread lives.
//! The parent watch (`ARGUS_COORD_PARENT`) runs the built binary and skips without it.

mod common;
mod hook_support;

use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use argus_coord::Env;
use hook_support::{Fixture, LEAD, UI, hook_with};
use serde_json::{Value, json};

/// What the stand-in answers on a stream: the events to send, then it holds the stream open
/// (`None`: hold it open, silent; an empty text: end the stream at once).
type Answer = Arc<dyn Fn(usize) -> Option<String> + Send + Sync>;

struct Server {
    port: u16,
    paths: Arc<Mutex<Vec<String>>>,
}

fn request_path(stream: &TcpStream) -> String {
    let mut reader = BufReader::new(stream);
    let mut first = String::new();
    reader.read_line(&mut first).ok();
    let mut line = String::new();
    while reader.read_line(&mut line).is_ok_and(|n| n > 2) {
        line.clear();
    }
    first
        .split_whitespace()
        .nth(1)
        .unwrap_or_default()
        .to_owned()
}

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

impl Server {
    fn start(answer_with: Answer) -> Server {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let paths = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&paths);
        thread::spawn(move || {
            for (n, stream) in listener.incoming().flatten().enumerate() {
                seen.lock().unwrap().push(request_path(&stream));
                let events = answer_with(n);
                thread::spawn(move || answer(stream, events));
            }
        });
        Server { port, paths }
    }

    fn paths(&self) -> Vec<String> {
        self.paths.lock().unwrap().clone()
    }
}

fn wake(data: &Value) -> String {
    format!("event: wake\ndata: {data}\n\n")
}

fn always(events: Option<String>) -> Answer {
    Arc::new(move |_| events.clone())
}

fn fixture(server: &Server) -> Fixture {
    let f = Fixture::new();
    let config = f.home.dashboard_home().join("config.json");
    fs::write(config, format!(r#"{{"port":{}}}"#, server.port)).unwrap();
    f
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

fn listen(f: &Fixture, sid: &str, event: &str, extra: &[(&str, &str)]) -> common::Ran {
    let input = json!({ "session_id": sid, "cwd": f.cwd_text(), "hook_event_name": event });
    hook_with(env_of(f, extra), "listen", &input.to_string(), &f.cwd)
}

#[test]
fn a_wake_for_the_thread_ends_the_listener_with_exit_2_and_the_line_on_stderr() {
    let server = Server::start(always(Some(wake(
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
    let server = Server::start(always(Some(wake(&data))));
    let f = fixture(&server);
    assert_eq!(
        listen(&f, UI, "Stop", &[]).err,
        "WAKE ui: a; b (3 wakes). Run: coord inbox\n"
    );
    let bad = Server::start(always(Some("event: wake\ndata: not json\n\n".to_owned())));
    let f = fixture(&bad);
    let ran = listen(&f, UI, "Stop", &[]);
    assert_eq!(
        (ran.code, ran.err.as_str()),
        (2, "WAKE ui: see coord inbox\n")
    );
}

#[test]
fn a_newer_listener_replaces_this_one_which_ends_silently_with_exit_0() {
    let server = Server::start(always(Some("event: replaced\ndata: {}\n\n".to_owned())));
    let f = fixture(&server);
    let ran = listen(&f, UI, "Stop", &[]);
    assert_eq!((ran.code, ran.out.as_str(), ran.err.as_str()), (0, "", ""));
}

#[test]
fn a_lead_listens_for_every_thread_of_its_project() {
    let server = Server::start(always(Some(wake(
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
    let server = Server::start(always(None));
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
    let server = Server::start(answer_with);
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
    let server = Server::start(always(Some(wake(
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

#[test]
fn session_start_says_wakes_are_automatic_only_where_the_settings_carry_the_listen_hook() {
    let server = Server::start(always(None));
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
    let server = Server::start(always(None));
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
