//! E2: coord dashboard hands the line to a real fake program and ends with its exit code.

// test-tag: e2e

use std::fs;

use crate::common::{Home, coord};
use crate::remote::env_plus;

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
#[ignore = "e2e"]
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
