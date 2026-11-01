//! Ports the `coord inbox` cases of `inbox.wake.test.mjs` (the message is queued, and the
//! dashboard is asked, best effort, to wake the thread: reason `thread-message`, text
//! `<sender>: <first 100 chars>`; a dashboard that is down never fails the verb; a message to
//! yourself or to a thread that does not exist wakes nobody) and of `owner.text.test.mjs` (`coord
//! inbox` prints the whole text of an owner post, and `inbox send` keeps its cap of 240). The
//! wake stream itself (listeners, replay) is the server's, ported in
//! `crates/argus-server/tests/it/wake_http.rs`.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::work_board::Work;
use serde_json::{Value, json};

/// A dashboard that answers 200 to everything and keeps the requests it was sent.
struct Dashboard {
    seen: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

/// The bytes a header line `content-length` promises after the blank line, 0 when absent.
fn promised(head: &str) -> usize {
    let lower = head.to_lowercase();
    let line = lower.lines().find(|l| l.starts_with("content-length:"));
    line.and_then(|l| l.split(':').nth(1))
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0)
}

fn read_request(stream: &mut std::net::TcpStream) -> String {
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
    let mut text = String::new();
    let mut chunk = [0_u8; 2048];
    while let Ok(n) = stream.read(&mut chunk) {
        text.push_str(&String::from_utf8_lossy(&chunk[..n]));
        let done = text
            .split_once("\r\n\r\n")
            .is_some_and(|(head, body)| body.len() >= promised(head));
        if n == 0 || done {
            break;
        }
    }
    text
}

impl Dashboard {
    fn start(work: &Work) -> Dashboard {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        listener.set_nonblocking(true).expect("nonblocking");
        let port = listener.local_addr().expect("addr").port();
        let config = work.home.dashboard_home().join("config.json");
        fs::write(config, json!({ "port": port }).to_string()).expect("config");
        let seen = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let (log, halt) = (Arc::clone(&seen), Arc::clone(&stop));
        let handle = thread::spawn(move || {
            while !halt.load(Ordering::SeqCst) {
                let Ok((mut stream, _)) = listener.accept() else {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                };
                stream.set_nonblocking(false).ok();
                let request = read_request(&mut stream);
                log.lock().expect("log").push(request);
                let body = "{}";
                let reply = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(reply.as_bytes()).ok();
            }
        });
        Dashboard {
            seen,
            stop,
            handle: Some(handle),
        }
    }

    /// The bodies of the wakes it was asked for.
    fn wakes(&self) -> Vec<Value> {
        let seen = self.seen.lock().expect("log");
        seen.iter()
            .filter(|r| r.starts_with("POST /api/p/alpha/wakes"))
            .filter_map(|r| {
                r.split_once("\r\n\r\n")
                    .and_then(|(_, b)| serde_json::from_str(b).ok())
            })
            .collect()
    }
}

impl Drop for Dashboard {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            handle.join().ok();
        }
    }
}

fn inbox_of(work: &Work, thread: &str) -> Vec<Value> {
    work.board()["inbox"][thread]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

#[test]
fn with_no_dashboard_running_the_message_is_queued_and_the_verb_succeeds() {
    let work = Work::new();
    let ran = work.lead(&["inbox", "send", "worker", "build is ready"]);
    assert_eq!(ran.code, 0, "{}{}", ran.out, ran.err);
    assert!(ran.out.contains("queued for worker"), "{}", ran.out);
    assert_eq!(
        inbox_of(&work, "worker").last().map(|m| m["text"].clone()),
        Some(json!("build is ready"))
    );
    assert_eq!(ran.err, "");
}

#[test]
fn with_the_dashboard_running_the_thread_is_woken_with_sender_and_the_first_100_chars() {
    let work = Work::new();
    let dashboard = Dashboard::start(&work);
    let long = "y".repeat(130);
    let ran = work.lead(&["inbox", "send", "worker", &long]);
    assert_eq!(ran.code, 0, "{}{}", ran.out, ran.err);
    let wakes = dashboard.wakes();
    assert_eq!(wakes.len(), 1, "{wakes:?}");
    assert_eq!(
        wakes[0],
        json!({"thread": "worker", "reason": "thread-message", "text": format!("coordinator: {}", "y".repeat(100))})
    );
    assert_eq!(
        inbox_of(&work, "worker").last().map(|m| m["text"].clone()),
        Some(json!(long)),
        "the inbox keeps the whole text"
    );
}

#[test]
fn a_message_to_yourself_or_to_a_thread_that_does_not_exist_wakes_nobody() {
    let work = Work::new();
    let dashboard = Dashboard::start(&work);
    assert_eq!(
        work.worker(&["inbox", "send", "worker", "to myself"]).code,
        0
    );
    let bad = work.lead(&["inbox", "send", "nobody", "x"]);
    assert_eq!(bad.code, 1, "{}", bad.out);
    assert!(dashboard.wakes().is_empty(), "{:?}", dashboard.wakes());
}

#[test]
fn inbox_prints_the_whole_text_of_an_owner_post_and_send_keeps_its_cap() {
    let work = Work::new();
    let text = "Steps: open the page.\n- first **bold** item\n- second item\n".repeat(500);
    let text = text.trim().to_owned();
    let mut board = work.board();
    board["inbox"]["worker"] = json!([{
        "id": "m-1790000000000-own1", "ts": "2026-10-01T12:00:00.000Z", "from": "human",
        "kind": "followup", "text": text, "ref": null, "deliveredAt": null,
    }]);
    work.save(&board);
    let out = work.worker(&["inbox"]);
    assert_eq!(out.code, 0, "{}", out.err);
    assert!(out.out.contains(&text), "all of the text is in the output");
    assert!(
        !out.out.contains('\u{2026}'),
        "nothing is shortened with an ellipsis"
    );
    assert!(
        out.out.contains("(m-1790000000000-own1)"),
        "the message id is printed"
    );
    let over = work.lead(&["inbox", "send", "worker", &"z".repeat(241)]);
    assert_ne!(over.code, 0);
    assert!(
        over.err.contains("241 chars, the limit is 240"),
        "{}",
        over.err
    );
    assert_eq!(
        work.lead(&["inbox", "send", "worker", &"z".repeat(240)])
            .code,
        0
    );
}
