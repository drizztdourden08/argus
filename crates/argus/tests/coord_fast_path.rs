//! The `argus coord` fast path of the real binary: help, an unknown verb, and exit 120 for a
//! verb that is not ported (no output, nothing written, hook JSON on stdin left alone). The
//! homes are temp folders; the real `~/.coord` is never named.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn argus(args: &[&str], home: &Path, stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_argus"))
        .arg("coord")
        .args(args)
        .env("COORD_HOME", home.join("coord"))
        .env("DASHBOARD_HOME", home.join("dash"))
        .env_remove("CLAUDE_CODE_SESSION_ID")
        .env_remove("COORD_PROJECT")
        .current_dir(home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // The child may exit without reading; a closed pipe is not an error here.
    child.stdin.take().unwrap().write_all(stdin.as_bytes()).ok();
    child.wait_with_output().unwrap()
}

fn entries(dir: &Path) -> usize {
    fs::read_dir(dir).map_or(0, Iterator::count)
}

#[test]
fn help_prints_the_usage_and_exits_0() {
    let home = tempfile::tempdir().unwrap();
    let out = argus(&["--help"], home.path(), "");
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.starts_with("Usage: coord <verb> ... [--project <key>] [--as <thread>]\n"));
    assert!(text.ends_with("coord hook session-start|prompt|tool|guard|stop\n"));
    assert_eq!(out.stderr, Vec::<u8>::new());
}

#[test]
fn an_unknown_verb_exits_1_with_the_usage_on_stderr() {
    let home = tempfile::tempdir().unwrap();
    let out = argus(&["frobnicate"], home.path(), "");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(out.stdout, Vec::<u8>::new());
    let text = String::from_utf8(out.stderr).unwrap();
    assert!(text.starts_with("Unknown verb \"frobnicate\".\n\nUsage: coord"));
}

#[test]
fn an_unported_verb_exits_120_with_no_output_and_no_file_written() {
    let home = tempfile::tempdir().unwrap();
    let json = r#"{"session_id":"s-1","cwd":"."}"#;
    for args in [
        &["hook", "tool"][..],
        &["wake-fault", "list"][..],
        &["proc", "list"][..],
        &["ping", "hello"][..],
    ] {
        let out = argus(args, home.path(), json);
        assert_eq!(out.status.code(), Some(120), "{args:?}");
        assert!(out.stdout.is_empty(), "{args:?} printed on stdout");
        assert!(out.stderr.is_empty(), "{args:?} printed on stderr");
    }
    assert_eq!(entries(&home.path().join("coord")), 0);
    assert_eq!(entries(&home.path().join("dash")), 0);
    assert_eq!(
        entries(home.path()),
        0,
        "nothing is created in the working directory"
    );
}
