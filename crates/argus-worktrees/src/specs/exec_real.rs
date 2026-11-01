//! KEEP: S3 (a program that hangs, or leaves a child holding its pipe, never holds up the answer).
//!
//! The one place a process is spawned, run against real programs. The limit under test is a real time limit, so these
//! specs are end to end: they start a child process and wait for it to be cut.

// test-tag: e2e

use crate::exec::{Spec, run_command};
use std::time::{Duration, Instant};

#[cfg(windows)]
fn sleeper() -> (&'static str, Vec<String>) {
    ("ping", ["-n", "8", "127.0.0.1"].map(String::from).to_vec())
}

#[cfg(not(windows))]
fn sleeper() -> (&'static str, Vec<String>) {
    ("sleep", vec!["8".to_string()])
}

#[test]
#[ignore = "e2e"]
fn a_program_that_hangs_is_stopped_after_the_timeout() {
    let (program, args) = sleeper();
    let started = Instant::now();
    let out = run_command(&Spec {
        program,
        args: &args,
        cwd: ".",
        env: &[],
        timeout: Duration::from_millis(150),
    });
    assert!(out.timed_out);
    assert!(out.error.unwrap().contains("timed out"));
    assert!(started.elapsed() < Duration::from_secs(5));
}

/// A program that ends at once and leaves a child behind that keeps its output pipe open for 8 seconds.
#[cfg(windows)]
fn leaves_a_child() -> (&'static str, Vec<String>) {
    let args = ["/d", "/c", "echo said& start /b ping -n 8 127.0.0.1"];
    ("cmd", args.map(String::from).to_vec())
}

#[cfg(not(windows))]
fn leaves_a_child() -> (&'static str, Vec<String>) {
    let args = ["-c", "echo said; sleep 8 &"];
    ("sh", args.map(String::from).to_vec())
}

#[test]
#[ignore = "e2e"]
fn a_child_left_behind_holding_the_pipe_does_not_hold_up_the_answer() {
    let (program, args) = leaves_a_child();
    let started = Instant::now();
    let out = run_command(&Spec {
        program,
        args: &args,
        cwd: ".",
        env: &[],
        timeout: Duration::from_secs(30),
    });
    assert_eq!(out.code, Some(0), "{out:?}");
    assert!(out.stdout.contains("said"), "{out:?}");
    assert!(
        started.elapsed() < Duration::from_secs(6),
        "answered after {:?}",
        started.elapsed()
    );
}
