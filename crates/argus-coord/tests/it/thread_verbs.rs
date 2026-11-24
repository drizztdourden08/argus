//! K2: the lines and records of coord thread and board.

// test-tag: critical

use crate::thread_support::Cli;
use serde_json::{Value, json};

fn fixture() -> Cli {
    Cli::new("work", &["--project", "alpha"])
}

fn register_parser(fx: &Cli) {
    fx.ok(&[
        "thread",
        "register",
        "worker-one",
        "--subject",
        "Parser",
        "--session",
        "Alpha - Parser",
        "--root",
        "somewhere",
    ]);
}

fn with_board(fx: &Cli, f: impl FnOnce(&mut Value)) {
    let mut board = fx.board();
    f(&mut board);
    fx.home.write_board("alpha", &board);
}

fn started() -> Cli {
    let fx = fixture();
    fx.ok(&["board", "init", "--name", "Alpha"]);
    let root = fx.cwd.to_string_lossy().into_owned();
    fx.ok(&[
        "thread",
        "register",
        "lead",
        "--role",
        "lead",
        "--root",
        &root,
        "--session",
        "Alpha",
    ]);
    fx.ok(&[
        "thread",
        "register",
        "worker",
        "--subject",
        "work",
        "--root",
        &root,
        "--session",
        "Alpha - Work",
    ]);
    fx
}

#[test]
fn board_init_creates_the_board_and_a_second_init_says_where_it_is() {
    let fx = fixture();
    let first = fx.ok(&["board", "init", "--name", "  Alpha  "]);
    assert!(
        first.out.starts_with("board created for alpha: "),
        "{}",
        first.out
    );
    assert!(
        first
            .out
            .contains("Next:\n  1. coord thread register lead --role lead")
    );
    assert!(
        first.out.contains("(the page: http://127.0.0.1:"),
        "{}",
        first.out
    );
    let board = fx.board();
    assert_eq!(board["name"], "Alpha");
    assert_eq!(board["schema"], 2);
    let second = fx.ok(&["board", "init"]);
    assert!(
        second.out.contains("board for alpha already exists at "),
        "{}",
        second.out
    );
    let forced = fx.ok(&["board", "init", "--force", "--name", "Beta"]);
    assert!(forced.out.starts_with("board created for alpha"));
    assert_eq!(fx.board()["name"], "Beta");
}

#[test]
fn a_thread_registers_updates_and_is_listed() {
    let fx = fixture();
    fx.ok(&["board", "init", "--name", "Alpha"]);
    let first = fx.ok(&[
        "thread",
        "register",
        "Worker One",
        "--subject",
        "Parser",
        "--session",
        "Alpha - Parser",
        "--root",
        "somewhere",
    ]);
    assert!(
        first
            .out
            .contains("registered thread worker-one (root somewhere)"),
        "{}",
        first.out
    );
    assert!(
        first.out.contains("tip: no session link found"),
        "{}",
        first.out
    );
    let thread = fx.board()["threads"]["worker-one"].clone();
    assert_eq!(thread["status"], "active");
    assert_eq!(thread["subject"], "Parser");
    assert_eq!(thread["rank"], 1024);
    assert_eq!(thread["rev"], 2);
}

#[test]
fn the_thread_doc_keeps_the_key_order_of_the_node_cli() {
    let fx = fixture();
    fx.ok(&["board", "init", "--name", "Alpha"]);
    register_parser(&fx);
    let thread = fx.board()["threads"]["worker-one"].clone();
    let keys: Vec<&str> = thread
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        [
            "id",
            "name",
            "role",
            "flags",
            "note",
            "subject",
            "brief",
            "registeredAt",
            "createdBy",
            "updatedAt",
            "updatedBy",
            "rev",
            "rank",
            "root",
            "session",
            "status",
            "lastSeen"
        ]
    );
}

#[test]
fn registering_again_updates_and_the_list_shows_it() {
    let fx = fixture();
    fx.ok(&["board", "init", "--name", "Alpha"]);
    register_parser(&fx);
    let again = fx.ok(&["thread", "register", "worker-one", "--note", "hello"]);
    assert!(
        again.out.starts_with("updated thread worker-one"),
        "{}",
        again.out
    );
    let list = fx.ok(&["thread", "list"]);
    assert!(
        list.out
            .starts_with("thread      status  flags  subject  seen"),
        "{}",
        list.out
    );
    assert!(
        list.out
            .contains("worker-one  active         Parser   0s ago  hello"),
        "{}",
        list.out
    );
}

#[test]
fn register_refuses_what_it_cannot_take() {
    let fx = fixture();
    fx.refused(
        &["thread", "register", "x"],
        "no board for project \"alpha\"",
    );
    fx.ok(&["board", "init"]);
    fx.refused(&["thread", "register"], "name text is required.");
    fx.refused(
        &["thread", "register", "!!!"],
        "thread name must contain letters or digits.",
    );
    fx.refused(
        &["thread", "register", &"n".repeat(41)],
        "name is 41 chars, the limit is 40. Shorten it.",
    );
    fx.refused(
        &["thread", "register", "ok", "--link", "nope"],
        "--link takes the session link from get_session (claude://...).",
    );
    fx.refused(
        &["thread", "register", "ok", "--subject", &"s".repeat(41)],
        "subject is 41 chars",
    );
    fx.refused(&["thread", "frobnicate"], "thread verb must be one of register, note, status, flag, done, list, show, set, add, rm, pause, resume, archive, unarchive.");
    let naming = fx.ok(&[
        "thread",
        "register",
        "lead",
        "--role",
        "lead",
        "--session",
        "Wrong",
    ]);
    assert!(naming.out.contains("naming: rename this session to \"alpha\" (set_session_title), then register again with that --session"), "{}", naming.out);
    let words = fx.ok(&[
        "thread",
        "register",
        "w",
        "--session",
        "alpha - one two three four five",
    ]);
    assert!(
        words
            .out
            .contains("naming: keep the subject to four words at most: \"alpha - <Subject>\""),
        "{}",
        words.out
    );
}

#[test]
fn a_thread_keeps_its_note_status_and_flags_and_ends() {
    let fx = started();
    let note = fx.ok(&["thread", "note", "parsing", "--as", "worker"]);
    assert_eq!(note.out, "worker: parsing\n");
    assert_eq!(fx.board()["threads"]["worker"]["note"], "parsing");
    fx.refused(
        &["thread", "note", &"n".repeat(101), "--as", "worker"],
        "note is 101 chars, the limit is 100. Shorten it.",
    );
    fx.refused(
        &["thread", "status", "sleeping", "--as", "worker"],
        "status must be one of dispatched, active, blocked, waiting, idle, paused, done.",
    );
    assert_eq!(
        fx.ok(&["thread", "status", "blocked", "--as", "worker"])
            .out,
        "worker: blocked\n"
    );
    assert_eq!(
        fx.ok(&["thread", "flag", "needs-human", "--as", "worker"])
            .out,
        "worker: needs-human set\n"
    );
    assert_eq!(fx.board()["threads"]["worker"]["flags"]["needsHuman"], true);
    assert_eq!(
        fx.ok(&["thread", "flag", "needs-human", "--clear", "--as", "worker"])
            .out,
        "worker: needs-human cleared\n"
    );
    fx.refused(
        &["thread", "flag", "bogus", "--as", "worker"],
        "flag must be one of needs-human, blocked.",
    );
    let list = fx.ok(&["thread", "list"]);
    assert!(list.out.contains("worker  blocked"), "{}", list.out);
    assert_eq!(
        fx.ok(&["thread", "done", "--as", "worker"]).out,
        "worker: done\n"
    );
    assert_eq!(fx.board()["threads"]["worker"]["status"], "done");
    let shown = fx.ok(&["thread", "show", "worker"]);
    assert!(
        shown.out.starts_with("{\n  \"id\": \"worker\","),
        "{}",
        shown.out
    );
    fx.refused(&["thread", "show", "ghost"], "no thread \"ghost\".");
}

#[test]
fn set_changes_what_the_caller_may_change() {
    let fx = started();
    let set = fx.ok(&[
        "thread",
        "set",
        "--subject",
        "parser",
        "--note",
        "n",
        "--status",
        "idle",
        "--as",
        "worker",
    ]);
    assert!(
        set.out
            .starts_with("worker: set note, status, subject (rev "),
        "{}",
        set.out
    );
    fx.refused(
        &["thread", "set", "--as", "worker"],
        "nothing to set: pass --subject, --brief, --note, --status, --before or --after.",
    );
    fx.refused(
        &["thread", "set", "--brief", "b", "--as", "worker"],
        "only the lead can change a thread brief or the thread order.",
    );
    fx.refused(
        &["thread", "set", "--status", "nope", "--as", "worker"],
        "status must be one of",
    );
    fx.refused(
        &[
            "thread", "set", "--if-rev", "x", "--note", "a", "--as", "worker",
        ],
        "--if-rev takes the revision number you last saw.",
    );
    fx.refused(
        &["thread", "set", "lead", "--note", "mine", "--as", "worker"],
        "worker cannot set lead: only lead itself or the lead can.",
    );
}

#[test]
fn the_lead_sets_a_brief_and_the_order_and_checks_revisions() {
    let fx = started();
    let lead = fx.ok(&[
        "thread", "set", "worker", "--brief", "B", "--before", "lead", "--as", "lead",
    ]);
    assert!(
        lead.out.contains("worker: set brief, order (rev "),
        "{}",
        lead.out
    );
    fx.refused(
        &[
            "thread", "set", "worker", "--after", "ghost", "--as", "lead",
        ],
        "no thread \"ghost\".",
    );
    let stale = fx.run(&[
        "thread",
        "set",
        "worker",
        "--if-rev",
        "1",
        "--subject",
        "late",
        "--as",
        "lead",
    ]);
    assert_eq!(stale.code, 1);
    assert!(
        stale.err.contains("worker changed since rev 1 (by "),
        "{}",
        stale.err
    );
}

#[test]
fn pause_resume_archive_and_unarchive_a_thread() {
    let fx = started();
    let paused = fx.ok(&["thread", "pause", "worker", "--as", "lead"]);
    assert_eq!(
        paused.out,
        "worker: paused. Its inbox tells it what to do; wake it if it is idle.\n"
    );
    assert_eq!(
        fx.ok(&["thread", "pause", "worker", "--as", "lead"]).out,
        "worker is already paused.\n"
    );
    assert_eq!(fx.board()["inbox"]["worker"].as_array().unwrap().len(), 1);
    assert!(
        fx.ok(&["thread", "resume", "worker", "--as", "lead"])
            .out
            .contains("worker: resumed.")
    );
    assert_eq!(
        fx.ok(&["thread", "resume", "worker", "--as", "lead"]).out,
        "worker is already not paused.\n"
    );
    fx.refused(
        &["thread", "archive", "lead", "--as", "lead"],
        "the lead thread is never archived.",
    );
    let archived = fx.ok(&[
        "thread", "archive", "worker", "--reason", "gone", "--as", "lead",
    ]);
    assert!(
        archived
            .out
            .starts_with("worker: archived. Off the thread list"),
        "{}",
        archived.out
    );
    assert!(!fx.ok(&["thread", "list"]).out.contains("worker"));
    assert!(fx.ok(&["thread", "list", "--all"]).out.contains("worker"));
    assert_eq!(
        fx.ok(&["thread", "archive", "worker", "--as", "lead"]).out,
        "worker is already archived.\n"
    );
    let back = fx.ok(&["thread", "unarchive", "worker", "--as", "lead"]);
    assert!(
        back.out.starts_with("worker: unarchived (active)."),
        "{}",
        back.out
    );
}

#[test]
fn locks_pass_to_the_next_when_a_thread_ends() {
    let fx = started();
    with_board(&fx, |b| {
        b["locks"] = json!({"db": {"resource": "db", "holder": "worker", "queue": ["lead"], "since": "2026-10-01T00:00:00.000Z"}});
    });
    let done = fx.ok(&["thread", "done", "--as", "worker"]);
    assert_eq!(
        done.out,
        "worker: done\nlock db passed to lead: message them (session \"Alpha\") so they start.\n"
    );
    assert_eq!(fx.board()["locks"]["db"]["holder"], "lead");
}

#[test]
fn dispatch_records_the_thread_and_prints_the_chip_prompt() {
    let fx = started();
    let out = fx.ok(&[
        "dispatch",
        "docs-two",
        "--subject",
        "Writing docs now",
        "--brief",
        "Write them.",
        "--worktree",
        "wt",
        "--as",
        "lead",
    ]);
    assert!(out.out.starts_with("dispatched docs-two (writing-docs-now). Chip title: \"Alpha - Writing Docs Now\". Prompt for the session chip:\n\nYou are \"Alpha - Writing Docs Now\", the \"docs-two\" thread of the Alpha coordination board (subject: writing-docs-now).\n"), "{}", out.out);
    assert!(
        out.out.contains("--project alpha  (from the wt worktree)"),
        "{}",
        out.out
    );
    assert!(out.out.ends_with("## Brief\nWrite them.\n"), "{}", out.out);
    let thread = fx.board()["threads"]["docs-two"].clone();
    assert_eq!(thread["status"], "dispatched");
    assert_eq!(thread["worktree"], "wt");
    assert_eq!(
        thread["note"],
        "dispatched, waiting for the session to register"
    );
}

#[test]
fn dispatch_refuses_a_duplicate_a_thread_caller_and_a_missing_brief() {
    let fx = started();
    fx.ok(&[
        "dispatch",
        "docs-two",
        "--subject",
        "x",
        "--brief",
        "y",
        "--as",
        "lead",
    ]);
    fx.refused(
        &[
            "dispatch",
            "docs-two",
            "--subject",
            "x",
            "--brief",
            "y",
            "--as",
            "lead",
        ],
        "thread \"docs-two\" already exists (dispatched).",
    );
    fx.refused(
        &[
            "dispatch",
            "other",
            "--subject",
            "x",
            "--brief",
            "y",
            "--as",
            "worker",
        ],
        "only the lead adds a thread (register with --role lead).",
    );
    fx.refused(
        &["dispatch", "other", "--subject", "x", "--as", "lead"],
        "brief text is required.",
    );
}

#[test]
fn thread_add_is_dispatch_with_a_custom_chip_title() {
    let fx = started();
    let via_thread = fx.ok(&[
        "thread",
        "add",
        "third",
        "--subject",
        "t",
        "--brief",
        "b",
        "--title",
        "custom words go here today",
        "--as",
        "lead",
    ]);
    assert!(
        via_thread
            .out
            .contains("Chip title: \"Alpha - Custom Words Go Here\""),
        "{}",
        via_thread.out
    );
}

#[test]
fn board_name_prefixes_show_and_the_retired_pages() {
    let fx = started();
    assert_eq!(
        fx.ok(&["board", "name", "Alpha Tool"]).out,
        "alpha is shown as \"Alpha Tool\". Sessions: the lead \"Alpha Tool\" and \"Alpha Tool - <Subject>\".\n"
    );
    fx.refused(
        &["board", "name"],
        "pass the project display name, 30 characters at most (e.g. \"RotP\").",
    );
    assert_eq!(
        fx.ok(&["board", "prefixes", "A", "B", "A"]).out,
        "thread titles on alpha start with: \"A - <Subject>\", \"B - <Subject>\".\n"
    );
    assert_eq!(
        fx.ok(&["board", "prefixes", "--clear"]).out,
        "thread titles on alpha start with: \"Alpha Tool - <Subject>\".\n"
    );
    fx.refused(
        &["board", "prefixes", &"x".repeat(31)],
        "each thread-title prefix is 30 characters at most.",
    );
    assert!(
        fx.ok(&["board", "page"])
            .out
            .starts_with("retired: the dashboard serves this page at http://127.0.0.1:")
    );
    assert!(
        fx.ok(&["board", "path"])
            .out
            .ends_with("alpha\\board.json\n")
            || fx
                .ok(&["board", "path"])
                .out
                .ends_with("alpha/board.json\n")
    );
    fx.refused(
        &["board", "frobnicate"],
        "board verb must be one of id, init, open, page, url, show, path, name, prefixes, shortcut, kanban.",
    );
    let json = fx.ok(&["board", "show", "--json"]);
    let summary: Value = serde_json::from_str(&json.out).expect("json");
    assert_eq!(summary["project"], "alpha");
    assert_eq!(summary["threads"].as_array().unwrap().len(), 2);
    assert!(summary["dirty"].as_u64().is_some());
    let text = fx.ok(&["board", "show"]);
    assert!(
        text.out.starts_with("# alpha  (http://127.0.0.1:"),
        "{}",
        text.out
    );
    assert!(text.out.contains("\n\nitems: 0   open questions: -   pending decisions: -   tests: 0   handed build: -   manual actions: -\n"), "{}", text.out);
}

#[test]
fn opening_the_board_waits_for_the_dashboard_verb() {
    let fx = started();
    let ran = fx.run(&["board", "open"]);
    assert_eq!(ran.code, 120);
    assert_eq!(ran.out, "");
    assert_eq!(ran.err, "");
}
