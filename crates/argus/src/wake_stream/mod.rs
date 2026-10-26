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

/// Where the command writes: stdout lines (which may fail) and status lines.
pub struct Io {
    /// Writes one wake line to stdout.
    pub out: Box<dyn FnMut(&str) -> io::Result<()> + Send>,
    /// Writes one status line to stderr.
    pub err: Box<dyn FnMut(&str) + Send>,
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
    err: Box<dyn FnMut(&str) + Send>,
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
struct Sink<'a> {
    out: &'a mut (dyn FnMut(&str) -> io::Result<()> + Send),
    stop: &'a CancellationToken,
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

impl Sink<'_> {
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

fn target(args: &WakeArgs) -> String {
    let label = args
        .id
        .as_deref()
        .map(|id| format!("&id={}", encode(id)))
        .unwrap_or_default();
    format!(
        "/api/p/{}/wakes/stream?thread={}&v=2{label}",
        args.project,
        encode(&args.thread)
    )
}

/// One connection, from the connect to the end of the stream. Returns whether it connected.
async fn attempt(
    args: &WakeArgs,
    at: &Endpoint,
    status: &mut Status,
    sink: &mut Sink<'_>,
) -> bool {
    let target = target(args);
    let request = Request {
        port: at.port,
        target: &target,
        token: at.token.as_deref(),
    };
    let stop = sink.stop.clone();
    let opened = tokio::select! {
        o = Connection::open(&request) => o,
        () = stop.cancelled() => return false,
    };
    let mut conn = match opened {
        Ok(conn) => conn,
        Err(e) => {
            let what = if status.state == State::Up {
                "connection lost"
            } else {
                "dashboard not reachable"
            };
            status.note(State::Down, &format!("{what} ({}); retrying", cause(&e)));
            return false;
        }
    };
    status.note(State::Up, "connected");
    let ended = tokio::select! {
        r = pump(&mut conn, |action| sink.on(action)) => Some(r),
        () = stop.cancelled() => None,
    };
    match ended {
        Some(Ok(Ended::ByServer)) => status.note(State::Down, "the stream ended; reconnecting"),
        Some(Err(e)) => status.note(
            State::Down,
            &format!("connection lost ({}); retrying", cause(&e)),
        ),
        Some(Ok(Ended::ByCaller)) | None => {}
    }
    true
}

/// Runs the command until `stop` rings or the stream is replaced. `argv` is the command line
/// after `argus` (it may start with `wake-stream`). Returns the exit code.
pub async fn run(argv: &[String], paths: &Paths, io: Io, stop: CancellationToken) -> u8 {
    let Io { mut out, err } = io;
    let args = match parse_wake_args(argv) {
        Ok(args) => args,
        Err(message) => {
            let mut err = err;
            err(&format!("argus wake-stream: {message}"));
            return 2;
        }
    };
    let mut status = Status {
        state: State::Init,
        err,
    };
    (status.err)(&format!(
        "wake-stream: project {}, thread {}",
        args.project, args.thread
    ));
    let mut sink = Sink {
        out: &mut *out,
        stop: &stop,
        reason: None,
    };
    let mut backoff = BACKOFF_START_MS;
    while !stop.is_cancelled() {
        match endpoint(paths) {
            None => status.note(
                State::Down,
                "no dashboard address known (is it installed?); retrying",
            ),
            Some(at) => {
                if attempt(&args, &at, &mut status, &mut sink).await {
                    backoff = BACKOFF_START_MS;
                }
            }
        }
        tokio::select! {
            () = tokio::time::sleep(Duration::from_millis(backoff)) => {}
            () = stop.cancelled() => {}
        }
        backoff = next_backoff(backoff);
    }
    if let Some(reason) = sink.reason {
        (status.err)(&format!("wake-stream: ending, {reason}"));
    }
    0
}
