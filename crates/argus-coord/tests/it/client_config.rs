//! K1: the dashboard port and token the client takes from the files, and the event stream reader.

// test-tag: critical

use std::fs;
use std::io::{self, Cursor, Read};

use crate::common::Home;
use argus_coord::client::{DEFAULT_PORT, SseReader, board_port, dashboard_config, server_port};
use serde_json::json;

fn write_config(home: &Home, config: &serde_json::Value) {
    fs::write(
        home.dashboard_home().join("config.json"),
        config.to_string(),
    )
    .unwrap();
}

#[test]
fn the_port_is_the_servers_own_pid_file_then_config_then_the_default() {
    let home = Home::new();
    let paths = home.env().paths(home.path());
    assert_eq!(dashboard_config(&paths), None, "no config.json: no config");
    assert_eq!(server_port(&paths), DEFAULT_PORT);

    write_config(&home, &json!({ "port": 51999 }));
    assert_eq!(dashboard_config(&paths).unwrap().port, 51999);
    assert_eq!(server_port(&paths), 51999);

    fs::write(
        home.dashboard_home().join("server.pid"),
        json!({"pid": 1, "port": 40123}).to_string(),
    )
    .unwrap();
    assert_eq!(dashboard_config(&paths).unwrap().port, 40123);
    assert_eq!(server_port(&paths), 40123);

    write_config(&home, &json!({}));
    fs::write(
        home.dashboard_home().join("server.pid"),
        json!({"pid": 1, "port": -3}).to_string(),
    )
    .unwrap();
    assert_eq!(dashboard_config(&paths).unwrap().port, DEFAULT_PORT);
}

#[test]
fn the_token_is_used_only_when_config_says_require_token() {
    let home = Home::new();
    let paths = home.env().paths(home.path());
    write_config(&home, &json!({ "port": 1, "token": "secret" }));
    assert_eq!(dashboard_config(&paths).unwrap().token, None);
    write_config(
        &home,
        &json!({ "port": 1, "token": "secret", "requireToken": true }),
    );
    assert_eq!(
        dashboard_config(&paths).unwrap().token.as_deref(),
        Some("secret")
    );
    write_config(&home, &json!({ "port": 1, "requireToken": true }));
    assert_eq!(dashboard_config(&paths).unwrap().token, None);
    write_config(&home, &json!({ "installDir": "X:/dash" }));
    assert_eq!(
        dashboard_config(&paths).unwrap().install_dir.as_deref(),
        Some("X:/dash")
    );
}

#[test]
fn the_board_port_is_the_env_port_overridden_by_config_else_the_default() {
    let home = Home::new();
    let paths = home.env().paths(home.path());
    assert_eq!(
        board_port(&home.env_with("DASHBOARD_PORT", "0"), &paths),
        DEFAULT_PORT
    );
    assert_eq!(
        board_port(&home.env_with("DASHBOARD_PORT", "4000"), &paths),
        4000
    );
    write_config(&home, &json!({ "port": 5000 }));
    assert_eq!(
        board_port(&home.env_with("DASHBOARD_PORT", "4000"), &paths),
        5000
    );
    write_config(&home, &json!({ "port": "6000" }));
    assert_eq!(
        board_port(&home.env_with("DASHBOARD_PORT", "4000"), &paths),
        6000
    );
    write_config(&home, &json!({ "port": "junk" }));
    assert_eq!(
        board_port(&home.env_with("DASHBOARD_PORT", "4000"), &paths),
        4000
    );
}

/// Hands out one byte per read, the way a slow stream does.
struct Trickle(Cursor<Vec<u8>>);

impl Read for Trickle {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(&mut buf[..1])
    }
}

fn events(text: &str) -> Vec<(String, String)> {
    let mut reader = SseReader::new(Trickle(Cursor::new(text.as_bytes().to_vec())));
    let mut out = Vec::new();
    while let Some(e) = reader.next_event().unwrap() {
        out.push((e.name, e.data));
    }
    out
}

#[test]
fn events_end_at_a_blank_line_and_the_name_defaults_to_message() {
    let got = events("event: wake\ndata: {\"lines\":[\"a\"]}\n\ndata: plain\n\n");
    assert_eq!(
        got,
        [
            ("wake".to_owned(), "{\"lines\":[\"a\"]}".to_owned()),
            ("message".to_owned(), "plain".to_owned()),
        ]
    );
}

#[test]
fn data_lines_are_joined_with_nothing_between_and_other_lines_are_ignored() {
    let got =
        events(": heartbeat\nid: 4\nevent: wake\ndata: one\ndata: two\ndata:three\nretry: 5\n\n");
    assert_eq!(got, [("wake".to_owned(), "onetwo".to_owned())]);
}

#[test]
fn a_partial_block_at_the_end_of_the_stream_is_not_an_event() {
    assert_eq!(
        events("event: wake\ndata: x\n"),
        Vec::<(String, String)>::new()
    );
    assert_eq!(
        events("event: replaced\n\n"),
        [("replaced".to_owned(), String::new())]
    );
}

#[test]
fn multi_byte_text_split_across_reads_survives() {
    let got = events("data: h\u{e9}llo \u{1F600}\n\n");
    assert_eq!(
        got,
        [("message".to_owned(), "h\u{e9}llo \u{1F600}".to_owned())]
    );
}
