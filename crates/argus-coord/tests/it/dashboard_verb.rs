//! `coord dashboard <verb> ...`: with no verb it prints the usage line; with one it hands the
//! rest of the line to the dashboard command (`ARGUS_EXE`, else this executable) as the positional
//! words and then each option as `--key value`, and ends with the code of that command.

use std::fs;

use crate::common::{Home, coord};
use crate::remote::{env_plus, ok};

/// A program that writes its arguments to the file `ARGS_OUT` names and ends with code 3.
fn fake_exe(home: &Home) -> std::path::PathBuf {
    let (name, body) = if cfg!(windows) {
        (
            "fake-dashboard.cmd",
            "@echo off\r\necho %*> \"%ARGS_OUT%\"\r\nexit /b 3\r\n",
        )
    } else {
        (
            "fake-dashboard.sh",
            "#!/bin/sh\necho \"$@\" > \"$ARGS_OUT\"\nexit 3\n",
        )
    };
    let path = home.path().join(name);
    fs::write(&path, body).expect("fake dashboard");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("chmod");
    }
    path
}

#[test]
fn with_no_verb_it_prints_the_usage_line() {
    let home = Home::new();
    let ran = coord(&["dashboard"], home.env(), home.path());
    let line = "coord dashboard <start|stop|status|open [<key>]|logs|resync|mcp|wake-stream|install-mcp|autostart> ...   (see dashboard/README.md)\n";
    assert_eq!(ok(&ran), line);
}

#[test]
fn it_hands_the_words_over_positionals_first_then_each_option_and_passes_the_exit_code_on() {
    let home = Home::new();
    let (exe, out) = (fake_exe(&home), home.path().join("args.txt"));
    let env = env_plus(
        &home.env(),
        &[
            ("ARGUS_EXE", &exe.to_string_lossy()),
            ("ARGS_OUT", &out.to_string_lossy()),
        ],
    );
    let argv = [
        "dashboard",
        "--tag",
        "a",
        "open",
        "--flag",
        "alpha",
        "--tag",
        "b",
        "--x=v",
    ];
    let ran = coord(&argv, env, home.path());
    assert_eq!(ran.code, 3, "{}", ran.err);
    let seen = fs::read_to_string(&out).expect("the arguments");
    assert_eq!(
        seen.split_whitespace().collect::<Vec<_>>(),
        [
            "open", "--tag", "a", "--tag", "b", "--flag", "alpha", "--x", "v"
        ]
    );
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
