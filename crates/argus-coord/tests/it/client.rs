//! S2: the dashboard client against a tiny in-process server on a loopback port: the request it
//! sends, the errors it maps, the health check and the autostart refusals.

// test-tag: standard

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::common::Home;
use argus_coord::client::{
    Client, ClientError, DashboardConfig, SseReader, dashboard_client, healthy,
};
use serde_json::json;

/// Answers each connection with the next `(status line, body)`, and returns the request heads.
fn serve(answers: Vec<(&'static str, String)>) -> (u16, JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let mut seen = Vec::new();
        for (status, body) in answers {
            let (mut stream, _) = listener.accept().unwrap();
            seen.push(read_request(&mut stream));
            let reply = format!(
                "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(reply.as_bytes()).unwrap();
        }
        seen
    });
    (port, handle)
}

/// The head and the body of one request, as text.
fn read_request(stream: &mut TcpStream) -> String {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut data = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        let n = stream.read(&mut chunk).unwrap();
        data.extend_from_slice(&chunk[..n]);
        let text = String::from_utf8_lossy(&data).into_owned();
        if let Some((head, body)) = text.split_once("\r\n\r\n") {
            let wanted = head
                .to_lowercase()
                .lines()
                .find_map(|l| {
                    l.strip_prefix("content-length: ")
                        .and_then(|v| v.trim().parse::<usize>().ok())
                })
                .unwrap_or(0);
            if body.len() >= wanted || n == 0 {
                return text;
            }
        }
        if n == 0 {
            return text;
        }
    }
}

fn client(port: u16, token: Option<&str>) -> Client {
    Client::new(&DashboardConfig {
        port,
        token: token.map(str::to_owned),
        install_dir: None,
    })
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn write_config(home: &Home, config: &serde_json::Value) {
    fs::write(
        home.dashboard_home().join("config.json"),
        config.to_string(),
    )
    .unwrap();
}

#[test]
fn a_get_returns_the_json_and_sends_no_credential_without_a_token() {
    let (port, seen) = serve(vec![("200 OK", r#"{"ok":true,"n":[1,2]}"#.to_owned())]);
    let got = client(port, None).get("/api/health").unwrap();
    assert_eq!(got, json!({"ok": true, "n": [1, 2]}));
    let head = seen.join().unwrap().remove(0).to_lowercase();
    assert!(head.starts_with("get /api/health http/1.1"), "{head}");
    assert!(!head.contains("authorization"), "{head}");
}

#[test]
fn the_bearer_token_is_sent_when_there_is_one() {
    let (port, seen) = serve(vec![("200 OK", "{}".to_owned())]);
    client(port, Some("tok")).get("/api/x").unwrap();
    let head = seen.join().unwrap().remove(0).to_lowercase();
    assert!(head.contains("authorization: bearer tok"), "{head}");
}

#[test]
fn a_post_sends_its_object_as_json() {
    let (port, seen) = serve(vec![("200 OK", r#"{"id":"R-1"}"#.to_owned())]);
    let got = client(port, None)
        .post("/api/p/alpha/runs", &json!({"name": "build"}))
        .unwrap();
    assert_eq!(got["id"], "R-1");
    let request = seen.join().unwrap().remove(0);
    let lower = request.to_lowercase();
    assert!(lower.starts_with("post /api/p/alpha/runs "), "{request}");
    assert!(
        lower.contains("content-type: application/json"),
        "{request}"
    );
    assert!(request.ends_with(r#"{"name":"build"}"#), "{request}");
}

#[test]
fn a_body_less_write_still_says_it_is_json() {
    let (port, seen) = serve(vec![("200 OK", String::new())]);
    let got = client(port, None)
        .request("DELETE", "/api/x", None)
        .unwrap();
    assert_eq!(got, serde_json::Value::Null);
    let head = seen.join().unwrap().remove(0).to_lowercase();
    assert!(head.starts_with("delete /api/x "), "{head}");
    assert!(head.contains("content-type: application/json"), "{head}");
}

#[test]
fn an_error_status_carries_the_servers_code_and_message() {
    let body = json!({"error": {"code": "not_found", "message": "no run R-9"}}).to_string();
    let (port, _seen) = serve(vec![
        ("404 Not Found", body),
        ("500 Internal Server Error", "oops".to_owned()),
    ]);
    let c = client(port, None);
    let err = c.get("/api/runs/R-9").unwrap_err();
    assert_eq!(
        err,
        ClientError::Http {
            status: 404,
            code: "not_found".into(),
            message: "no run R-9".into()
        }
    );
    assert_eq!(err.to_string(), "no run R-9");
    let plain = c.get("/api/x").unwrap_err();
    assert_eq!(
        plain,
        ClientError::Http {
            status: 500,
            code: "http_error".into(),
            message: "500 Internal Server Error".into()
        }
    );
}

#[test]
fn a_server_that_does_not_answer_is_the_down_error_with_its_cause() {
    let err = client(free_port(), None).get("/api/health").unwrap_err();
    assert_eq!(err, ClientError::Down(Some("ECONNREFUSED".into())));
    assert_eq!(
        err.to_string(),
        "the dashboard is not running (dashboard start): ECONNREFUSED"
    );
    assert_eq!(
        ClientError::down(None).to_string(),
        "the dashboard is not running (dashboard start)"
    );
}

#[test]
fn health_is_true_only_for_a_success_status() {
    let (up, _a) = serve(vec![("200 OK", "{}".to_owned())]);
    assert!(healthy(&client(up, None)));
    let (sick, _b) = serve(vec![("503 Service Unavailable", "{}".to_owned())]);
    assert!(!healthy(&client(sick, None)));
    assert!(!healthy(&client(free_port(), None)));
}

#[test]
fn a_healthy_server_is_used_without_starting_anything() {
    let home = Home::new();
    let (port, seen) = serve(vec![("200 OK", "{}".to_owned())]);
    write_config(&home, &json!({ "port": port }));
    let paths = home.env().paths(home.path());
    let got = dashboard_client(&home.env(), &paths, true).unwrap();
    assert_eq!(got.base, format!("http://127.0.0.1:{port}"));
    assert_eq!(seen.join().unwrap().len(), 1);
}

#[test]
fn without_autostart_a_server_that_is_down_is_the_plain_down_error() {
    let home = Home::new();
    write_config(&home, &json!({ "port": free_port() }));
    let paths = home.env().paths(home.path());
    let err = dashboard_client(&home.env(), &paths, false).unwrap_err();
    assert_eq!(err, ClientError::Down(None));
}

#[test]
fn with_autostart_but_no_install_to_start_it_says_so() {
    let home = Home::new();
    write_config(&home, &json!({ "port": free_port() }));
    let paths = home.env().paths(home.path());
    let env = home.env_with(
        "COORD_DASHBOARD_INSTALL",
        &home.path().join("nowhere").to_string_lossy(),
    );
    let err = dashboard_client(&env, &paths, true).unwrap_err();
    assert_eq!(
        err,
        ClientError::Down(Some("no dashboard install found".into()))
    );
}

#[test]
fn a_stream_from_the_server_is_read_as_events() {
    let body = "event: wake\ndata: {\"n\":1}\n\nevent: replaced\ndata: \n\n".to_owned();
    let (port, seen) = serve(vec![("200 OK", body)]);
    let stream = client(port, None)
        .stream(
            "/api/p/alpha/wakes/stream?thread=ui&v=2&id=x",
            Duration::from_secs(5),
        )
        .unwrap();
    let mut reader = SseReader::new(stream);
    let first = reader.next_event().unwrap().unwrap();
    assert_eq!(
        (first.name.as_str(), first.data.as_str()),
        ("wake", "{\"n\":1}")
    );
    assert_eq!(reader.next_event().unwrap().unwrap().name, "replaced");
    assert!(reader.next_event().unwrap().is_none());
    let head = seen.join().unwrap().remove(0).to_lowercase();
    assert!(head.contains("accept: text/event-stream"), "{head}");
}
