//! K2: coord dashboard prints its usage line and refuses a command that cannot start.

// test-tag: critical

use crate::common::{Home, coord};
use crate::remote::{env_plus, ok};

#[test]
fn with_no_verb_it_prints_the_usage_line() {
    let home = Home::new();
    let ran = coord(&["dashboard"], home.env(), home.path());
    let line = "coord dashboard <start|stop|status|open [<key>]|logs|resync|mcp|wake-stream|install-mcp|autostart> ...   (see dashboard/README.md)\n";
    assert_eq!(ok(&ran), line);
}

#[test]
fn a_command_that_cannot_start_is_a_refusal() {
    let home = Home::new();
    let missing = home.path().join("no-such-exe");
    let env = env_plus(&home.env(), &[("ARGUS_EXE", &missing.to_string_lossy())]);
    let ran = coord(&["dashboard", "start"], env, home.path());
    assert_eq!(ran.code, 1);
    assert!(
        ran.err.starts_with("coord: could not run the dashboard: "),
        "{}",
        ran.err
    );
}
