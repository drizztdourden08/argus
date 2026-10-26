//! The pure parts of `dashboard wake-stream`: the arguments, the ONE line printed per wake (hooks and Claude
//! sessions parse it, so it must match byte for byte), reading the server-sent events and the reconnect pauses.

use argus_wake::sse::SseParser;
use argus_wake::wake_stream::{
    BACKOFF_MAX_MS, BACKOFF_START_MS, StreamAction, WakeArgs, action_of, line_of, next_backoff,
    parse_wake_args,
};
use serde_json::json;

fn parse(args: &[&str]) -> Result<WakeArgs, String> {
    let owned: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
    parse_wake_args(&owned)
}

fn args(project: &str, thread: &str, id: Option<&str>) -> WakeArgs {
    WakeArgs {
        project: project.to_owned(),
        thread: thread.to_owned(),
        id: id.map(str::to_owned),
    }
}

#[test]
fn arguments_are_parsed_with_a_star_thread_by_default_and_a_cleaned_label() {
    assert_eq!(
        parse(&["wake-stream", "--project", "rotp", "--thread", "ui"]),
        Ok(args("rotp", "ui", None))
    );
    assert_eq!(parse(&["--project=rotp"]), Ok(args("rotp", "*", None)));
    assert_eq!(
        parse(&["--project", "rotp", "--id", "  RotP \n\t lead "]),
        Ok(args("rotp", "*", Some("RotP lead")))
    );
    assert_eq!(
        parse(&["wake-stream", "--project", "rotp", "--other", "--thread=*"]),
        Ok(args("rotp", "*", None)),
        "an unknown flag is ignored"
    );
}

#[test]
fn a_missing_or_wrong_argument_says_what_is_wrong() {
    assert_eq!(
        parse(&["--project", "rotp", "--id"]),
        Err("--id needs a value".to_owned())
    );
    assert_eq!(
        parse(&["--project", "rotp", "--id", "--thread"]),
        Err("--id needs a value".to_owned())
    );
    assert_eq!(
        parse(&["--thread", "ui"]),
        Err("--project <key> is required (the project key, e.g. rotp)".to_owned())
    );
    assert_eq!(
        parse(&[]),
        Err("--project <key> is required (the project key, e.g. rotp)".to_owned())
    );
    assert_eq!(
        parse(&["--project", "Bad Key"]),
        Err("--project <key> is required (the project key, e.g. rotp)".to_owned())
    );
    assert_eq!(
        parse(&["--project", "a", "--thread", "x y"]),
        Err("--thread must be a thread id or *".to_owned())
    );
    assert_eq!(
        parse(&["--project", "a", "stray"]),
        Err("unexpected argument \"stray\"".to_owned())
    );
}

#[test]
fn the_printed_line_is_wake_thread_the_lines_joined_by_semicolons_and_the_inbox_command() {
    let lines = ["T-5 failed: no note", "T-6\nfailed:   x", "+3 more", "   "].map(str::to_owned);
    assert_eq!(
        line_of("ui", &lines),
        "WAKE ui: T-5 failed: no note; T-6 failed: x; +3 more Run: coord inbox"
    );
    assert_eq!(line_of("*", &[]), "WAKE *:  Run: coord inbox");
}

#[test]
fn a_wake_frame_prints_a_line_a_replaced_frame_ends_the_stream_and_anything_else_is_ignored() {
    let wake = json!({"id": "W-1", "scope": "alpha", "thread": "ui", "at": "x", "lines": ["owner note on I-1: A later note."], "count": 1});
    assert_eq!(
        action_of("wake", &wake),
        Some(StreamAction::Print(
            "WAKE ui: owner note on I-1: A later note. Run: coord inbox".to_owned()
        ))
    );
    assert_eq!(
        action_of("replaced", &json!({"reason": "newer listener"})),
        Some(StreamAction::Replaced)
    );
    assert_eq!(
        action_of(
            "ready",
            &json!({"scope": "alpha", "thread": "ui", "windowMs": 15000})
        ),
        None
    );
}

#[test]
fn frames_are_read_across_chunks_and_a_frame_that_is_not_json_or_has_no_data_is_skipped() {
    let mut parser = SseParser::new();
    let nothing = parser.push("event: ready\ndata: {\"scope\":\"alpha\"}");
    assert_eq!(nothing.len(), 0, "no blank line yet");
    let first = parser.push("\n\nevent: wake\ndata: {\"id\":\"W-1\",\n\n");
    assert_eq!(first.len(), 1);
    assert_eq!(
        (first[0].name.as_str(), first[0].data["scope"].as_str()),
        ("ready", Some("alpha"))
    );
    let rest =
        parser.push(": ping\n\nevent: wake\ndata: {\"id\":\"W-2\"}\n\ndata: {\"id\":\"W-3\"}\n\n");
    let seen: Vec<(String, String)> = rest
        .iter()
        .map(|f| {
            (
                f.name.clone(),
                f.data["id"].as_str().unwrap_or_default().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        seen,
        [
            ("wake".to_owned(), "W-2".to_owned()),
            ("message".to_owned(), "W-3".to_owned())
        ]
    );
}

#[test]
fn the_pause_before_a_reconnect_doubles_up_to_15_seconds() {
    let mut pause = BACKOFF_START_MS;
    let mut seen = vec![pause];
    for _ in 0..6 {
        pause = next_backoff(pause);
        seen.push(pause);
    }
    assert_eq!(seen, [1000, 2000, 4000, 8000, 15_000, 15_000, 15_000]);
    assert_eq!(BACKOFF_MAX_MS, 15_000);
}
