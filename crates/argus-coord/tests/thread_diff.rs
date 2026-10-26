//! Differential tests of the WP3 verbs (threads, board, purge, check, tick, sync import): the
//! same steps on the Node CLI and on `argus coord`, in twin temp homes seeded with the standard
//! fixture, comparing the exit code, stdout, stderr and every file (`board.json` byte for byte
//! after the harness's volatile rules). The read verbs come first, then the write verbs. A test
//! skips with a reason when node, `ai-config` or the `argus` binary is absent.
//!
//! Each row of a table is who runs it (`l` the lead, `w` the worker, `-` nobody, so `--as`
//! decides) and the words after `coord`; `--project alpha` is added unless the row names one.

mod common;
mod support;
mod thread_support;

use std::fs;

use support::differ::{Differ, Engines, Report, Step};
use thread_support::{Row, steps};

const NEWCOMER_SESSION: &str = "33333333-3333-4333-8333-333333333333";
const LONG_NOTE: &str = "nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn";

fn differ(test: &str) -> Option<Differ> {
    let engines = Engines::or_skip(test)?;
    Some(Differ::standard(engines).expect("twin homes"))
}

fn judge(report: &Report) {
    println!("{report}");
    report.assert_no_mismatch();
}

fn run(test: &str, rows: &[Row]) {
    let Some(differ) = differ(test) else {
        return;
    };
    let report = differ.run(&steps(&[rows])).expect("run the steps");
    judge(&report);
    assert_eq!(report.not_ported(), 0, "every WP3 verb is served\n{report}");
}

const READS: &[Row] = &[
    ("l", &["thread", "list"]),
    ("l", &["thread", "list", "--all"]),
    ("l", &["thread", "show"]),
    ("l", &["thread", "show", "worker"]),
    ("l", &["thread", "show", "nobody"]),
    ("l", &["thread"]),
    ("l", &["thread", "frobnicate"]),
    ("l", &["board", "show"]),
    ("l", &["board", "show", "--json"]),
    ("l", &["board", "path"]),
    ("l", &["board", "page"]),
    ("l", &["board", "url"]),
    ("l", &["board", "frobnicate"]),
    ("l", &["check"]),
    ("l", &["check", "--json"]),
    ("l", &["check", "--stale", "0"]),
    ("l", &["check", "--stale", "abc"]),
    ("l", &["tick"]),
    ("l", &["tick", "--dir", "x"]),
    ("l", &["trash"]),
    ("l", &["trash", "--json"]),
    ("w", &["trash"]),
    ("l", &["thread", "list", "--project", "nosuch"]),
    ("l", &["board", "show", "--project", "beta"]),
    ("l", &["sync", "export"]),
    ("l", &["sync", "frobnicate"]),
];

#[test]
fn read_verbs_thread_board_check_tick() {
    run("read_verbs_thread_board_check_tick", READS);
}

const WORKER_WRITES: &[Row] = &[
    ("w", &["thread", "note", "parsing the second file"]),
    ("w", &["thread", "note"]),
    ("w", &["thread", "note", LONG_NOTE]),
    ("w", &["thread", "status", "blocked"]),
    ("w", &["thread", "status", "sleeping"]),
    ("w", &["thread", "flag", "needs-human"]),
    ("w", &["thread", "flag", "blocked", "--clear"]),
    ("w", &["thread", "flag", "bogus"]),
    (
        "w",
        &[
            "thread",
            "set",
            "--subject",
            "parser",
            "--note",
            "n",
            "--status",
            "idle",
        ],
    ),
    ("w", &["thread", "set", "--brief", "new brief"]),
    ("w", &["thread", "set", "--status", "nope"]),
    ("w", &["thread", "set", "--if-rev", "x", "--note", "a"]),
    ("w", &["thread", "set"]),
    ("w", &["thread", "set", "lead", "--note", "mine now"]),
    (
        "w",
        &["thread", "set", "--if-rev", "1", "--subject", "late"],
    ),
    ("w", &["thread", "pause"]),
    ("w", &["thread", "pause"]),
    ("w", &["thread", "resume"]),
    ("w", &["thread", "resume"]),
    ("w", &["thread", "archive", "lead"]),
    ("w", &["thread", "archive", "--reason", "session gone"]),
    ("w", &["thread", "unarchive"]),
    ("w", &["thread", "done"]),
    ("w", &["thread", "list", "--all"]),
    ("w", &["thread", "show"]),
];

#[test]
fn thread_writes_as_the_worker() {
    run("thread_writes_as_the_worker", WORKER_WRITES);
}

const LEAD_WRITES: &[Row] = &[
    (
        "l",
        &[
            "thread",
            "add",
            "parser-two",
            "--subject",
            "parser two",
            "--brief",
            "Port the second parser.",
        ],
    ),
    (
        "l",
        &[
            "thread",
            "add",
            "parser-two",
            "--subject",
            "x",
            "--brief",
            "again",
        ],
    ),
    (
        "l",
        &[
            "dispatch",
            "docs",
            "--subject",
            "Docs",
            "--brief",
            "Write them.",
            "--worktree",
            "docs-tree",
            "--title",
            "the docs thread now here",
        ],
    ),
    ("l", &["dispatch", "docs", "--subject", "Docs"]),
    ("l", &["dispatch", "--subject", "Docs", "--brief", "x"]),
    ("w", &["dispatch", "w2", "--subject", "w", "--brief", "x"]),
    (
        "l",
        &[
            "thread",
            "set",
            "worker",
            "--brief",
            "A new brief",
            "--before",
            "lead",
        ],
    ),
    ("l", &["thread", "set", "worker", "--after", "ghost"]),
    (
        "l",
        &[
            "thread",
            "set",
            "worker",
            "--if-rev",
            "1",
            "--subject",
            "stale",
        ],
    ),
    ("l", &["thread", "pause", "worker"]),
    ("l", &["thread", "resume", "worker"]),
    (
        "l",
        &["thread", "archive", "worker", "--reason", "gone for now"],
    ),
    ("l", &["thread", "archive", "worker"]),
    ("l", &["thread", "archive", "lead"]),
    ("l", &["thread", "unarchive", "worker"]),
    ("l", &["thread", "list"]),
    ("l", &["check"]),
    ("l", &["tick", "woke", "worker", "docs"]),
    ("l", &["tick", "woke"]),
    ("l", &["tick"]),
];

#[test]
fn thread_writes_as_the_lead() {
    run("thread_writes_as_the_lead", LEAD_WRITES);
}

const REGISTER: &[Row] = &[
    (
        "w",
        &[
            "thread",
            "register",
            "worker",
            "--session",
            "alpha - Parser",
            "--note",
            "back",
        ],
    ),
    (
        "w",
        &[
            "thread",
            "register",
            "worker",
            "--session",
            "wrong title here",
        ],
    ),
    (
        "w",
        &[
            "thread",
            "register",
            "worker",
            "--session",
            "alpha - one two three four five",
        ],
    ),
    ("w", &["thread", "register", "worker", "--link", "bad link"]),
    (
        "w",
        &[
            "thread",
            "register",
            "worker",
            "--link",
            "claude://claude.ai/epitaxy/abc",
        ],
    ),
    ("w", &["thread", "register"]),
    ("w", &["thread", "register", "!!!"]),
    (
        "l",
        &[
            "thread",
            "register",
            "lead",
            "--session",
            "alpha",
            "--root",
            "elsewhere",
        ],
    ),
    ("l", &["thread", "register", "lead", "--session", "nope"]),
    ("w", &["thread", "list"]),
    ("w", &["thread", "show"]),
];

#[test]
fn thread_register_and_naming() {
    run("thread_register_and_naming", REGISTER);
}

#[test]
fn new_sessions_register_with_a_host_link_and_a_role() {
    let Some(differ) = differ("new_sessions_register_with_a_host_link_and_a_role") else {
        return;
    };
    let fresh = Step::new(&[
        "thread",
        "register",
        "Fresh Thread",
        "--subject",
        "Fresh",
        "--session",
        "alpha - Fresh",
        "--project",
        "alpha",
    ])
    .session(NEWCOMER_SESSION)
    .env("CLAUDE_CODE_HOST_SESSION_ID", "host-123");
    let again = Step::new(&["thread", "register", "fresh-thread", "--project", "alpha"])
        .session(NEWCOMER_SESSION);
    let beta_lead = Step::new(&[
        "thread",
        "register",
        "coordinator",
        "--role",
        "coordinator",
        "--session",
        "alpha",
        "--project",
        "beta",
    ])
    .session("44444444-4444-4444-8444-444444444444");
    let beta_thread = Step::new(&[
        "thread",
        "register",
        "someone",
        "--subject",
        "x",
        "--session",
        "beta - X",
        "--project",
        "beta",
    ])
    .session("55555555-5555-4555-8555-555555555555");
    let report = differ
        .run(&[fresh, again, beta_lead, beta_thread])
        .expect("run");
    judge(&report);
}

#[test]
fn a_placeholder_folds_into_the_registered_thread() {
    let session = "66666666-6666-4666-8666-666666666666";
    let prompt = format!(r#"{{"session_id":"{session}","hook_event_name":"UserPromptSubmit"}}"#);
    let Some(differ) = differ("a_placeholder_folds_into_the_registered_thread") else {
        return;
    };
    let on = |words: &[&str]| Step::new(words).session(session);
    let late = on(&[
        "thread",
        "register",
        "late-bloomer",
        "--subject",
        "late",
        "--session",
        "alpha - Late",
        "--project",
        "alpha",
    ]);
    let report = differ
        .run(&[
            on(&["hook", "prompt", "--project", "alpha"]).stdin(&prompt),
            on(&["thread", "list", "--all", "--project", "alpha"]),
            late,
            on(&["thread", "list", "--all", "--project", "alpha"]),
            on(&["board", "show", "--json", "--project", "alpha"]),
        ])
        .expect("run");
    judge(&report);
}

const LONG_NAME: &str = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";

const BOARD_WRITES: &[Row] = &[
    ("l", &["board", "name", "Alpha Tool"]),
    ("l", &["board", "name"]),
    ("l", &["board", "name", LONG_NAME]),
    ("l", &["board", "prefixes", "Alpha", "Tool", "Alpha"]),
    ("l", &["board", "prefixes", LONG_NAME]),
    ("l", &["board", "prefixes"]),
    ("l", &["board", "prefixes", "--clear"]),
    ("l", &["board", "init"]),
    ("l", &["board", "init", "--force", "--name", "  Forced  "]),
    (
        "l",
        &["board", "init", "--name", "Gamma", "--project", "gamma"],
    ),
    ("l", &["board", "show", "--project", "gamma"]),
    ("l", &["board", "shortcut"]),
    ("l", &["board", "shortcut"]),
    (
        "l",
        &[
            "board",
            "shortcut",
            "--thread",
            "worker",
            "--root",
            "tool-root",
        ],
    ),
    ("l", &["board", "shortcut", "--thread", "nobody"]),
    (
        "l",
        &[
            "board",
            "shortcut",
            "--thread",
            "worker",
            "--root",
            "missing-root",
        ],
    ),
    ("w", &["board", "shortcut"]),
];

#[test]
fn board_verbs_write_and_create() {
    let Some(differ) = differ("board_verbs_write_and_create") else {
        return;
    };
    for home in [&differ.twin().node, &differ.twin().rust] {
        fs::create_dir_all(home.work().join("tool-root")).expect("tool root");
    }
    let report = differ.run(&steps(&[BOARD_WRITES])).expect("run");
    judge(&report);
    assert_eq!(report.not_ported(), 0, "{report}");
}

const PURGE_DRY: &[Row] = &[
    ("l", &["purge", "thread", "worker"]),
    ("l", &["purge", "thread", "worker", "--reason", "cleanup"]),
    ("l", &["purge", "thread", "lead"]),
    ("l", &["purge", "thread", "ghost"]),
    ("l", &["purge", "thread"]),
    ("l", &["purge"]),
    ("l", &["purge", "--keep", "lead"]),
    ("l", &["purge", "--status", "done,archived"]),
    ("l", &["purge", "--status", "bogus"]),
    ("l", &["purge", "--idle", "0"]),
    ("l", &["purge", "--idle", "9999"]),
    ("l", &["purge", "--trash"]),
    ("l", &["purge", "--trash", "--older", "x"]),
    ("w", &["purge", "thread", "lead"]),
    ("l", &["purge", "thread", "worker", "--confirm", "wrong"]),
    ("l", &["purge", "thread", "worker", "--confirm"]),
    ("l", &["restore"]),
    ("l", &["restore", "--from", "nofile.json"]),
    (
        "l",
        &["restore", "--from", "nofile.json", "--confirm", "alpha"],
    ),
];

const PURGE_DO: &[Row] = &[
    ("l", &["trash"]),
    ("l", &["trash", "--for", "worker"]),
    ("l", &["trash", "--json"]),
    ("l", &["purge", "--trash", "--confirm", "alpha"]),
    ("l", &["purge", "--trash", "--confirm", "alpha"]),
    (
        "l",
        &[
            "purge",
            "thread",
            "worker",
            "--reason",
            "cleanup",
            "--confirm",
            "alpha",
        ],
    ),
    ("l", &["thread", "rm", "worker"]),
    ("l", &["thread", "list", "--all"]),
    ("l", &["board", "show", "--json"]),
];

#[test]
fn purge_trash_and_refusals() {
    let Some(differ) = differ("purge_trash_and_refusals") else {
        return;
    };
    let dry = differ.run(&steps(&[PURGE_DRY])).expect("run");
    judge(&dry);
    let soft_delete: &[Row] = &[("l", &["item", "rm", "I-3"])];
    let done = differ.run(&steps(&[soft_delete, PURGE_DO])).expect("run");
    judge(&done);
}

const SAVED: &[Row] = &[
    ("l", &["sync", "import", "saved"]),
    ("l", &["sync", "import", "saved"]),
    ("l", &["sync", "import", "nowhere"]),
    ("l", &["sync", "import"]),
    ("w", &["inbox"]),
    ("l", &["thread", "list"]),
    ("l", &["board", "show", "--json"]),
];

#[test]
fn sync_import_replays_saved_entries() {
    let Some(differ) = differ("sync_import_replays_saved_entries") else {
        return;
    };
    let entry = r#"{"id":"O-1","kind":"followup","replyTo":"Q-9","text":"hello from the owner","author":"human","thread":"worker","createdAt":"2026-10-02T00:00:00.000Z"}"#;
    let late = r#"{"id":"O-2","kind":"pause","replyTo":"worker","text":"","author":"human","createdAt":"2026-10-02T00:00:01.000Z"}"#;
    for home in [&differ.twin().node, &differ.twin().rust] {
        let dir = home.work().join("saved").join("entries");
        fs::create_dir_all(&dir).expect("entries folder");
        fs::write(dir.join("b.json"), late).expect("entry");
        fs::write(dir.join("a.json"), entry).expect("entry");
        fs::write(dir.join("notes.txt"), "not json").expect("other file");
    }
    let report = differ.run(&steps(&[SAVED])).expect("run");
    judge(&report);
}
