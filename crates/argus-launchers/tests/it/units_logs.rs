//! The pure pieces of the log path: redaction, plain text, line splitting and the size
//! window.
//!
//! Keep rule: K1, K4.
// test-tag: critical

use argus_launchers::{
    LineSplitter, LogWindow, MemoryStore, PendingLine, Redactor, RunLimits, RunRow, RunStore,
    Stream, plain_text,
};

use crate::common::new_run;

#[test]
fn the_redactor_replaces_known_values_first_ignores_short_ones_and_catches_patterns() {
    let redactor = Redactor::new(&["abcd1234", "ab", ""]);
    assert_eq!(redactor.redact("x abcd1234 y"), "x [redacted] y");
    assert_eq!(redactor.redact("ab stays"), "ab stays");
    assert_eq!(redactor.redact("TOKEN = s3cr3t"), "TOKEN = [redacted]");
    assert_eq!(
        redactor.redact("authorization: bearer XYZ"),
        "authorization: bearer [redacted]"
    );
    assert_eq!(redactor.redact("nothing here"), "nothing here");
    assert_eq!(Redactor::new::<&str>(&[]).redact("plain"), "plain");
}

#[test]
fn the_longest_known_secret_is_replaced_whole() {
    let redactor = Redactor::new(&["secret", "secret-and-more"]);
    assert_eq!(redactor.redact("a secret-and-more b"), "a [redacted] b");
}

#[test]
fn plain_text_drops_colour_cursor_and_control_sequences() {
    assert_eq!(plain_text("\u{1b}[36mline 12\u{1b}[0m"), "line 12");
    assert_eq!(plain_text("\u{1b}[2K\u{1b}[1Gdone"), "done");
    assert_eq!(plain_text("\u{1b}]0;title\u{7}text"), "text");
    assert_eq!(plain_text("a\u{0}b\u{7}c\td"), "abc\td");
}

#[test]
fn lines_end_at_lf_or_crlf_and_the_last_partial_line_waits() {
    let mut splitter = LineSplitter::new();
    assert_eq!(
        splitter.push(Stream::Out, "one\r\ntwo\n\nfour", 0),
        ["one", "two", ""]
    );
    assert!(splitter.has_partial());
    assert!(
        splitter.release(999, false).is_empty(),
        "not idle long enough"
    );
    let released = splitter.release(1000, false);
    assert_eq!(released, [(Stream::Out, vec!["four".to_owned()])]);
    assert!(!splitter.has_partial());
}

#[test]
fn a_carriage_return_keeps_the_last_segment_and_the_streams_do_not_mix() {
    let mut splitter = LineSplitter::new();
    assert_eq!(splitter.push(Stream::Out, "10%\r50%\r100%\n", 0), ["100%"]);
    assert_eq!(splitter.push(Stream::Out, "half", 0), Vec::<String>::new());
    assert_eq!(splitter.push(Stream::Err, "err\n", 0), ["err"]);
    assert_eq!(splitter.push(Stream::Out, " line\n", 0), ["half line"]);
    let forced = splitter.release(0, true);
    assert!(forced.is_empty());
}

#[test]
fn a_line_longer_than_16k_is_cut_the_same_whatever_the_chunking() {
    let mut splitter = LineSplitter::new();
    let mut pieces = splitter.push(Stream::Out, &"x".repeat(30_000), 0);
    pieces.extend(splitter.push(Stream::Out, &format!("{}\nafter\n", "x".repeat(10_000)), 0));
    let lengths: Vec<usize> = pieces.iter().map(|p| p.chars().count()).collect();
    assert_eq!(lengths, [16_384, 16_384, 7_232, 5]);
}

fn run_row() -> RunRow {
    MemoryStore::new().insert(new_run("R-1", "job"))
}

fn out(text: &str) -> PendingLine {
    PendingLine {
        stream: Stream::Out,
        at: "2026-01-01T00:00:00.000Z".to_owned(),
        text: text.to_owned(),
    }
}

fn tiny() -> RunLimits {
    RunLimits {
        log_cap_bytes: 100,
        log_head_bytes: 30,
        ..RunLimits::default()
    }
}

#[test]
fn the_window_keeps_the_head_and_drops_the_oldest_tail_lines_over_the_cap() {
    let (limits, mut window) = (tiny(), LogWindow::new(&run_row(), &tiny()));
    let lines: Vec<PendingLine> = (0..10).map(|_| out("0123456789")).collect();
    let first = window.plan(&limits, &lines);
    let heads: Vec<bool> = first.commit.lines.iter().map(|l| l.head).collect();
    assert_eq!(heads[..4], [true, true, true, false]);
    assert_eq!(first.commit.lines.last().unwrap().line.seq, 10);
    assert_eq!(
        (
            first.commit.drop_up_to,
            first.commit.log_dropped,
            first.commit.log_bytes
        ),
        (Some(4), 1, 99)
    );
    window.accept(&first);
    let second = window.plan(&limits, &[out("0123456789")]);
    assert_eq!(
        (
            second.commit.drop_up_to,
            second.commit.log_dropped,
            second.commit.log_bytes,
            second.commit.log_seq
        ),
        (Some(5), 2, 99, 11)
    );
}

#[test]
fn a_plan_that_is_not_accepted_leaves_the_window_as_it_was() {
    let (limits, window) = (tiny(), LogWindow::new(&run_row(), &tiny()));
    let lines: Vec<PendingLine> = (0..10).map(|_| out("0123456789")).collect();
    let lost = window.plan(&limits, &lines);
    let again = window.plan(&limits, &lines);
    assert_eq!(
        lost.commit, again.commit,
        "a failed commit loses the batch and nothing else"
    );
}

#[test]
fn last_line_is_the_programs_own_plain_text_clamped_and_never_a_system_line() {
    let (limits, window) = (
        RunLimits::default(),
        LogWindow::new(&run_row(), &RunLimits::default()),
    );
    let sys = PendingLine {
        stream: Stream::Sys,
        ..out("exited with code 0")
    };
    let plan = window.plan(&limits, &[out("\u{1b}[36m  hi  \u{1b}[0m"), sys.clone()]);
    assert_eq!(plan.commit.last_line.as_deref(), Some("hi"));
    let long = window.plan(&limits, &[out(&"y".repeat(201))]);
    let last = long.commit.last_line.unwrap();
    assert_eq!(last.chars().count(), 200);
    assert!(last.ends_with('~'));
    assert_eq!(window.plan(&limits, &[sys]).commit.last_line, None);
}
