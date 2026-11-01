//! The stand-in wake stream of the listener tests and the call of `coord hook listen`.

use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::{Fixture, StandIn, hook_with};
use crate::common::Ran;
use argus_coord::Env;
use serde_json::{Value, json};

/// What the stand-in answers on a stream: the events to send, then it holds the stream open
/// (`None`: hold it open, silent; an empty text: end the stream at once).
pub type Answer = Arc<dyn Fn(usize) -> Option<String> + Send + Sync>;

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

pub fn serve(answer_with: Answer) -> StandIn {
    StandIn::start(move |n, stream, _| {
        let events = answer_with(n);
        thread::spawn(move || answer(stream, events));
    })
}

pub fn wake(data: &Value) -> String {
    format!("event: wake\ndata: {data}\n\n")
}

pub fn always(events: Option<String>) -> Answer {
    Arc::new(move |_| events.clone())
}

pub fn env_of(f: &Fixture, extra: &[(&str, &str)]) -> Env {
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

pub fn listen(f: &Fixture, sid: &str, event: &str, extra: &[(&str, &str)]) -> Ran {
    let input = json!({ "session_id": sid, "cwd": f.cwd_text(), "hook_event_name": event });
    hook_with(env_of(f, extra), "listen", &input.to_string(), &f.cwd)
}
