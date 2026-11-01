//! `argus wake-stream` against a test server, ported from `wake.cli.test.mjs`: one stdout line
//! per wake and nothing else there, status on stderr, a reconnect after the server goes away and
//! comes back (an event nobody received is replayed to it), a newer stream for the same thread
//! replacing the older (which exits 0 and never reconnects), a broken stdout ending the stream
//! with code 0, and a missing `--project` exiting 2. The runs call the command's own `run` with
//! sinks that keep what it writes.

use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use argus::wake_stream::{Io, run};
use argus_core::Paths;
use argus_core::clock::now_ms;
use argus_testkit::{TestOptions, TestServer, make_home};
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

type Lines = Arc<Mutex<Vec<String>>>;
type Print = Box<dyn FnMut(&str) -> io::Result<()> + Send>;

struct Homes {
    coord: TempDir,
    dash: TempDir,
}

impl Homes {
    fn new() -> Self {
        let homes = Homes {
            coord: tempfile::tempdir().expect("coord home"),
            dash: tempfile::tempdir().expect("dashboard home"),
        };
        make_home(homes.coord.path());
        homes
    }

    fn paths(&self) -> Paths {
        Paths::new(self.coord.path(), Some(self.dash.path().to_path_buf()))
    }

    /// A server on these homes. The configured port is one nothing listens on, so a command
    /// that cannot find the live server never reaches the owner's real dashboard.
    async fn server(&self, window_ms: u64) -> TestServer {
        let closed = std::net::TcpListener::bind(("127.0.0.1", 0))
            .and_then(|l| l.local_addr())
            .map_or(1, |a| a.port());
        TestServer::start(TestOptions {
            coord_home: Some(self.coord_path()),
            dashboard_home: Some(self.dash.path().to_path_buf()),
            config: json!({ "port": closed, "wake": { "windowMs": window_ms }, "wakePolicy": { "debounceMs": 0 } }),
            ..TestOptions::default()
        })
        .await
    }

    fn coord_path(&self) -> PathBuf {
        self.coord.path().to_path_buf()
    }
}

/// A running command and what it wrote.
struct Running {
    out: Lines,
    err: Lines,
    stop: CancellationToken,
    done: JoinHandle<u8>,
}

fn keep(lines: &Lines) -> Lines {
    Arc::clone(lines)
}

fn push(lines: &Lines, line: &str) {
    lines
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(line.to_owned());
}

fn spawn_with(args: &[&str], paths: Paths, out: (Lines, Print)) -> Running {
    let errors = Lines::default();
    let sink = keep(&errors);
    let io = Io {
        out: out.1,
        err: Box::new(move |line| push(&sink, line)),
    };
    let stop = CancellationToken::new();
    let words: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
    let token = stop.clone();
    let done = tokio::spawn(async move { run(&words, &paths, io, token).await });
    Running {
        out: out.0,
        err: errors,
        stop,
        done,
    }
}

/// A command whose stdout keeps every line.
fn start(args: &[&str], paths: Paths) -> Running {
    let lines = Lines::default();
    let sink = keep(&lines);
    let out = Box::new(move |line: &str| {
        push(&sink, line);
        Ok(())
    });
    spawn_with(args, paths, (lines, out))
}

fn read(lines: &Lines) -> Vec<String> {
    lines.lock().unwrap_or_else(PoisonError::into_inner).clone()
}

async fn until(what: &str, ms: u64, check: impl Fn() -> bool) {
    let end = Instant::now() + Duration::from_millis(ms);
    while !check() {
        assert!(Instant::now() < end, "timed out waiting for {what}");
        tokio::time::sleep(Duration::from_millis(40)).await;
    }
}

async fn connected(run: &Running) {
    let err = keep(&run.err);
    until("the command to connect", 15_000, move || {
        read(&err).iter().any(|l| l.contains("connected"))
    })
    .await;
}

async fn post(t: &TestServer, path: &str, body: &Value) {
    let reply = t.write("POST", path, body).await;
    assert_eq!(reply.status, 200, "{path}: {}", reply.text());
}

async fn result(t: &TestServer, outcome: &str, text: &str) {
    let body = json!({ "outcome": outcome, "text": text });
    post(t, "/api/p/alpha/tests/T-2/result", &body).await;
}

async fn entry(t: &TestServer, extra: Value) {
    let mut body = json!({ "id": format!("O-{:x}-0001", now_ms()), "createdAt": argus_core::clock::now_iso(), "kind": "followup" });
    if let (Some(b), Some(e)) = (body.as_object_mut(), extra.as_object()) {
        b.extend(e.clone());
    }
    post(t, "/api/p/alpha/entries", &body).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn it_prints_one_line_per_coalesced_wake_and_reconnects_after_a_server_restart() {
    let homes = Homes::new();
    let server = homes.server(120).await;
    let cli = start(
        &["wake-stream", "--project", "alpha", "--thread", "ui"],
        homes.paths(),
    );
    connected(&cli).await;

    result(&server, "fail", "Error text missing").await;
    entry(&server, json!({ "replyTo": "Q-1", "text": "Yes." })).await;
    let out = keep(&cli.out);
    until("the first line", 8000, || !read(&out).is_empty()).await;
    tokio::time::sleep(Duration::from_millis(600)).await;
    assert_eq!(
        read(&cli.out),
        ["WAKE ui: T-2 failed: Error text missing; owner answered Q-1: Yes. Run: coord inbox"],
        "both wakes were inside one window: one line"
    );
    entry(
        &server,
        json!({ "item": "I-1", "thread": "ui", "text": "A later note." }),
    )
    .await;
    until("the second line", 8000, || read(&out).len() >= 2).await;
    assert_eq!(
        read(&cli.out)[1],
        "WAKE ui: owner note on I-1: A later note. Run: coord inbox"
    );

    server.stop(true).await;
    let err = keep(&cli.err);
    until("the command to notice", 15_000, || {
        read(&err)
            .iter()
            .any(|l| l.contains("reconnecting") || l.contains("retrying"))
    })
    .await;
    assert!(!cli.done.is_finished(), "it does not exit");

    let again = homes.server(120).await;
    result(&again, "blocked", "while you were away").await;
    until("the replayed line", 20_000, || read(&out).len() >= 3).await;
    let third = read(&cli.out)[2].clone();
    assert!(
        third.starts_with("WAKE ui: ")
            && third.contains("T-2 blocked: while you were away")
            && third.ends_with(" Run: coord inbox"),
        "{third}"
    );
    assert!(read(&cli.out).iter().all(|l| l.starts_with("WAKE ")));
    cli.stop.cancel();
    assert_eq!(cli.done.await.expect("the command ends"), 0);
    again.stop(true).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn without_a_project_it_says_what_is_missing_and_exits_2() {
    let homes = Homes::new();
    let cli = start(&["wake-stream"], homes.paths());
    assert_eq!(cli.done.await.expect("the command ends"), 2);
    assert!(
        read(&cli.err)
            .iter()
            .any(|l| l.contains("--project <key> is required"))
    );
    assert_eq!(read(&cli.out).len(), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_newer_stream_for_the_same_thread_replaces_it_and_the_older_exits_0_for_good() {
    let homes = Homes::new();
    let server = homes.server(80).await;
    let first = start(
        &["--project", "alpha", "--thread", "ui", "--id", "first"],
        homes.paths(),
    );
    let other = start(&["--project", "alpha", "--thread", "api"], homes.paths());
    connected(&first).await;
    connected(&other).await;
    let listed = server.get("/api/p/alpha/wakes/listeners").await.json();
    assert_eq!(listed.as_array().map(Vec::len), Some(2));

    let second = start(
        &["--project", "alpha", "--thread", "ui", "--id", "second"],
        homes.paths(),
    );
    assert_eq!(first.done.await.expect("the first ends"), 0);
    assert!(read(&first.out).is_empty(), "nothing on stdout");
    assert!(
        read(&first.err)
            .iter()
            .any(|l| l.contains("replaced by a newer wake-stream"))
    );
    connected(&second).await;
    tokio::time::sleep(Duration::from_millis(2500)).await;
    let open = server.get("/api/p/alpha/wakes/listeners").await.json();
    let mut labels: Vec<(String, Value)> = open
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|l| {
            (
                l["thread"].as_str().unwrap_or_default().to_owned(),
                l["label"].clone(),
            )
        })
        .collect();
    labels.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        labels,
        [
            ("api".to_owned(), Value::Null),
            ("ui".to_owned(), json!("second"))
        ],
        "one listener per thread, the newer one"
    );
    assert!(!second.done.is_finished() && !other.done.is_finished());

    result(&server, "fail", "only the newer hears this").await;
    let out = keep(&second.out);
    until("the wake line", 8000, || {
        read(&out).iter().any(|l| l.contains("T-2 failed"))
    })
    .await;
    for run in [&second, &other] {
        run.stop.cancel();
    }
    server.stop(true).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_stdout_that_cannot_be_written_to_ends_the_stream_with_code_0() {
    let homes = Homes::new();
    let server = homes.server(50).await;
    let writes = Lines::default();
    let sink = keep(&writes);
    let broken = Box::new(move |line: &str| {
        push(&sink, line);
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "write EPIPE"))
    });
    let args = [
        "wake-stream",
        "--project",
        "alpha",
        "--thread",
        "ui",
        "--id",
        "epipe",
    ];
    let cli = spawn_with(&args, homes.paths(), (Lines::default(), broken));
    connected(&cli).await;
    result(&server, "blocked", "nobody reads").await;
    let code = tokio::time::timeout(Duration::from_secs(10), cli.done)
        .await
        .expect("it ends")
        .expect("joined");
    assert_eq!(code, 0);
    assert_eq!(
        read(&writes).len(),
        1,
        "it stopped after the first failed write"
    );
    assert!(
        read(&cli.err)
            .iter()
            .any(|l| l.contains("stdout is closed (EPIPE)"))
    );
    tokio::time::sleep(Duration::from_millis(300)).await;
    let open = server.get("/api/p/alpha/wakes/listeners").await.json();
    assert_eq!(open, json!([]), "the connection was closed");
    server.stop(true).await;
}
