//! The top-level output of a call the Node shim forwarded (`ARGUS_FORWARDED_FROM` set) is the
//! Node `dashboard` bin's, ported from its `--version`, usage and unknown command behaviour; a
//! direct call keeps clap's own. Temp home, `DASHBOARD_PORT=0`, nothing is started.
//! E2, K2: the real binary answers a forwarded call with the Node text.
// test-tag: e2e

use std::process::{Command, Output};

use tempfile::TempDir;

const USAGE_HEAD: &str =
    "Usage: dashboard <command>\n  serve [--port n]            run the server in the foreground\n";
const USAGE_TAIL: &str = "  install-mcp [--desktop [--write]] [--code [--dir <project dir>]]\n";

fn run(home: &TempDir, forwarded: bool, args: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_argus"));
    command
        .args(args)
        .env("COORD_HOME", home.path())
        .env("DASHBOARD_HOME", home.path().join("dashboard"))
        .env("DASHBOARD_PORT", "0")
        .env_remove("ARGUS_FORWARDED_FROM");
    if forwarded {
        command.env("ARGUS_FORWARDED_FROM", "1234");
    }
    command.output().expect("argus runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
#[ignore = "e2e"]
fn a_forwarded_usage_is_the_node_text() {
    let home = tempfile::tempdir().expect("a temp home");
    for (args, code) in [
        (vec![], 2),
        (vec!["--help"], 0),
        (vec!["-h"], 0),
        (vec!["help"], 0),
    ] {
        let out = run(&home, true, &args);
        let stdout = text(&out.stdout);
        assert!(stdout.starts_with(USAGE_HEAD), "{stdout}");
        assert!(stdout.ends_with(USAGE_TAIL), "{stdout}");
        assert_eq!(stdout.lines().count(), 13, "{stdout}");
        assert_eq!(out.status.code(), Some(code), "{args:?}");
        assert_eq!(text(&out.stderr), "");
    }
}

#[test]
#[ignore = "e2e"]
fn a_direct_call_keeps_the_clap_output() {
    let home = tempfile::tempdir().expect("a temp home");
    let version = run(&home, false, &["--version"]);
    assert_eq!(
        text(&version.stdout),
        format!("argus {}\n", env!("CARGO_PKG_VERSION"))
    );
    let help = text(&run(&home, false, &["--help"]).stdout);
    assert!(help.contains("Usage: argus"), "{help}");
    let bad = run(&home, false, &["frobnicate"]);
    assert!(text(&bad.stderr).starts_with("argus: unknown command"));
    assert_eq!(bad.status.code(), Some(2));
}
