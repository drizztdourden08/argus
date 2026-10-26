//! `argus wake-stream --project <key> [--thread <id|*>] [--id <label>]`: stays connected to the
//! wake stream of a project and prints one line on stdout per wake,
//!
//! ```text
//! WAKE <thread>: <line1>; <line2> Run: coord inbox
//! ```
//!
//! and nothing else on stdout (status goes to stderr). It runs under the Monitor tool of a
//! Claude session: each line printed wakes that session, which then reads its inbox. The server
//! being down or restarting is not an error: the command reconnects forever with a growing
//! pause, starts nothing itself, and ends when told to (SIGINT, SIGTERM), when its reader goes
//! away (a broken pipe on stdout), or when the server says `replaced`: a newer wake-stream for
//! the same thread took over, so this one exits 0 and never reconnects. `--id` is a label the
//! server lists. The stream is asked for with `v=2`, which makes it unique per thread.

mod connection;
mod session;

use std::io;
use std::time::Duration;

use argus_core::Paths;
use argus_server::endpoint::{Endpoint, endpoint};
use argus_wake::wake_stream::{
    BACKOFF_START_MS, StreamAction, WakeArgs, next_backoff, parse_wake_args,
};
use tokio_util::sync::CancellationToken;

use connection::{Connection, Request};
use session::{Ended, Flow, pump};

/// Writes one wake line to stdout; fails when nobody reads it any more.
pub type Print = Box<dyn FnMut(&str) -> io::Result<()> + Send>;

/// Writes one status line to stderr.
pub type Say = Box<dyn FnMut(&str) + Send>;

/// Where the command writes: stdout lines (which may fail) and status lines.
pub struct Io {
    /// Wake lines.
    pub out: Print,
    /// Status lines.
    pub err: Say,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Init,
    Down,
    Up,
}

/// The state of the command, so a long outage prints one line and not one per retry.
struct Status {
    state: State,
    err: Say,
}

impl Status {
    fn note(&mut self, next: State, message: &str) {
        if self.state != next {
            (self.err)(&format!("wake-stream: {message}"));
        }
        self.state = next;
    }
}

/// Where the lines go, and why the command ended when it did.
struct Sink {
    out: Print,
    stop: CancellationToken,
    reason: Option<String>,
}

fn cause(e: &io::Error) -> String {
    match e.kind() {
        io::ErrorKind::BrokenPipe => "EPIPE".to_owned(),
        io::ErrorKind::ConnectionRefused => "ECONNREFUSED".to_owned(),
        io::ErrorKind::ConnectionReset => "ECONNRESET".to_owned(),
        _ => e.to_string(),
    }
}

impl Sink {
    fn end(&mut self, reason: String) -> Flow {
        self.reason = Some(reason);
        self.stop.cancel();
        Flow::Stop
    }

    fn on(&mut self, action: StreamAction) -> Flow {
        match action {
            StreamAction::Print(line) => match (self.out)(&line) {
                Ok(()) => Flow::Go,
                Err(e) => self.end(format!("stdout is closed ({})", cause(&e))),
            },
            StreamAction::Replaced => {
                self.end("replaced by a newer wake-stream for the same thread".to_owned())
            }
        }
    }
}

fn encode(text: &str) -> String {
    text.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&b) {
                char::from(b).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

fn target(wanted: &WakeArgs) -> String {
    let label = wanted
        .id
        .as_deref()
        .map(|id| format!("&id={}", encode(id)))
        .unwrap_or_default();
    format!(
        "/api/p/{}/wakes/stream?thread={}&v=2{label}",
        wanted.project,
        encode(&wanted.thread)
    )
}

/// The command: what it listens to, what it says, where its lines go.
struct Command {
    wanted: WakeArgs,
    status: Status,
    sink: Sink,
}

impl Command {
    /// Connects to the server; `None` when it cannot, or the command was stopped meanwhile.
    async fn open(&mut self, at: &Endpoint) -> Option<Connection> {
        let target = target(&self.wanted);
        let request = Request {
            port: at.port,
            target: &target,
            token: at.token.as_deref(),
        };
        let stop = self.sink.stop.clone();
        let opened = tokio::select! {
            o = Connection::open(&request) => o,
            () = stop.cancelled() => return None,
        };
        let lost = if self.status.state == State::Up {
            "connection lost"
        } else {
            "dashboard not reachable"
        };
        match opened {
            Ok(conn) => Some(conn),
            Err(e) => {
                let message = format!("{lost} ({}); retrying", cause(&e));
                self.status.note(State::Down, &message);
                None
            }
        }
    }

    /// Reads the stream to its end and says how it ended.
    async fn read(&mut self, mut conn: Connection) {
        self.status.note(State::Up, "connected");
        let stop = self.sink.stop.clone();
        let sink = &mut self.sink;
        let ended = tokio::select! {
            r = pump(&mut conn, |action| sink.on(action)) => Some(r),
            () = stop.cancelled() => None,
        };
        match ended {
            Some(Ok(Ended::ByServer)) => {
                self.status
                    .note(State::Down, "the stream ended; reconnecting");
            }
            Some(Err(e)) => {
                let message = format!("connection lost ({}); retrying", cause(&e));
                self.status.note(State::Down, &message);
            }
            Some(Ok(Ended::ByCaller)) | None => {}
        }
    }

    /// One try: find the server, connect, read. Returns whether it connected.
    async fn attempt(&mut self, paths: &Paths) -> bool {
        let Some(at) = endpoint(paths) else {
            let message = "no dashboard address known (is it installed?); retrying";
            self.status.note(State::Down, message);
            return false;
        };
        let Some(conn) = self.open(&at).await else {
            return false;
        };
        self.read(conn).await;
        true
    }
}

/// Runs the command until `stop` rings or the stream is replaced. `words` is the command line
/// after `argus` (it may start with `wake-stream`). Returns the exit code.
pub async fn run(words: &[String], paths: &Paths, io: Io, stop: CancellationToken) -> u8 {
    let Io { out, mut err } = io;
    let wanted = match parse_wake_args(words) {
        Ok(wanted) => wanted,
        Err(message) => {
            err(&format!("argus wake-stream: {message}"));
            return 2;
        }
    };
    err(&format!(
        "wake-stream: project {}, thread {}",
        wanted.project, wanted.thread
    ));
    let sink = Sink {
        out,
        stop: stop.clone(),
        reason: None,
    };
    let status = Status {
        state: State::Init,
        err,
    };
    let mut command = Command {
        wanted,
        status,
        sink,
    };
    let mut backoff = BACKOFF_START_MS;
    while !stop.is_cancelled() {
        if command.attempt(paths).await {
            backoff = BACKOFF_START_MS;
        }
        tokio::select! {
            () = tokio::time::sleep(Duration::from_millis(backoff)) => {}
            () = stop.cancelled() => {}
        }
        backoff = next_backoff(backoff);
    }
    if let Some(reason) = command.sink.reason.take() {
        (command.status.err)(&format!("wake-stream: ending, {reason}"));
    }
    0
}
