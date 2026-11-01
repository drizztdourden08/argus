//! E3: parity of done, archive, purge and restore against the Node CLI.

// test-tag: e2e

use std::fs;

use crate::support::differ::{Differ, Engines};
use crate::thread_support::{Row, steps};

const SEED: &[Row] = &[
    (
        "w",
        &["item", "add", "Extra work", "--ac", "one", "--ac", "two"],
    ),
    ("w", &["item", "add", "Second"]),
    (
        "w",
        &[
            "ask",
            "I-4",
            "Which way?",
            "--context",
            "This is a long enough context sentence for the rule.",
            "--option",
            "left",
            "--option",
            "right",
        ],
    ),
    ("w", &["lock", "claim", "parser-lock"]),
    ("l", &["lock", "claim", "parser-lock"]),
    ("w", &["action", "add", "do the manual thing"]),
    ("w", &["log", "did some stuff"]),
    ("w", &["test", "add", "I-4", "check it", "--mode", "manual"]),
    ("w", &["thread", "note", "working"]),
];

const ENDING: &[Row] = &[
    ("l", &["check"]),
    ("l", &["board", "show"]),
    ("w", &["thread", "done"]),
    ("l", &["check", "--json"]),
    (
        "l",
        &[
            "thread",
            "register",
            "worker",
            "--session",
            "alpha - Parser",
        ],
    ),
    ("w", &["lock", "claim", "parser-lock"]),
    ("l", &["thread", "archive", "lead"]),
    ("l", &["lock", "release", "parser-lock"]),
    ("l", &["thread", "archive", "worker"]),
    ("l", &["tick"]),
    ("l", &["board", "show"]),
    ("l", &["trash"]),
];

const PURGE: &[Row] = &[
    ("l", &["purge", "thread", "worker"]),
    (
        "l",
        &[
            "purge",
            "thread",
            "worker",
            "--confirm",
            "alpha",
            "--reason",
            "tidy up",
        ],
    ),
    ("l", &["board", "show", "--json"]),
    ("l", &["inbox"]),
];

const RESTORE: &[Row] = &[
    (
        "l",
        &[
            "restore",
            "--from",
            "backup.json",
            "--confirm",
            "alpha",
            "--thread",
            "ghost",
        ],
    ),
    (
        "l",
        &["restore", "--from", "backup.json", "--confirm", "wrong"],
    ),
    (
        "l",
        &[
            "restore",
            "--from",
            "backup.json",
            "--confirm",
            "alpha",
            "--thread",
            "worker",
        ],
    ),
    (
        "l",
        &["restore", "--from", "backup.json", "--confirm", "alpha"],
    ),
    ("l", &["board", "show", "--json"]),
    ("l", &["check", "--json"]),
    ("l", &["purge", "--keep", "lead"]),
    (
        "l",
        &[
            "purge",
            "--status",
            "done,placeholder",
            "--confirm",
            "alpha",
        ],
    ),
    ("l", &["purge", "--idle", "1", "--confirm", "alpha"]),
    ("l", &["purge", "--keep", "lead", "--confirm", "alpha"]),
    (
        "l",
        &["restore", "--from", "backup.json", "--confirm", "alpha"],
    ),
    ("l", &["board", "show", "--json"]),
];

fn differ(test: &str) -> Option<Differ> {
    let engines = Engines::or_skip(test)?;
    Some(Differ::standard(engines).expect("twin homes"))
}

#[test]
#[ignore = "e2e"]
fn done_and_archive_pass_the_locks_on() {
    let Some(differ) = differ("done_and_archive_pass_the_locks_on") else {
        return;
    };
    let report = differ.run(&steps(&[SEED, ENDING])).expect("run");
    println!("{report}");
    report.assert_no_mismatch();
}

fn copy_backup(differ: &Differ) {
    for home in [&differ.twin().node, &differ.twin().rust] {
        let dir = home.coord().join("alpha").join("purged");
        let backup = fs::read_dir(&dir)
            .expect("purged folder")
            .flatten()
            .map(|e| e.path())
            .find(|p| p.extension().is_some_and(|x| x == "json"))
            .expect("a backup file");
        fs::copy(backup, home.work().join("backup.json")).expect("copy the backup");
    }
}

#[test]
#[ignore = "e2e"]
fn purge_a_thread_with_records_then_restore_it() {
    let Some(differ) = differ("purge_a_thread_with_records_then_restore_it") else {
        return;
    };
    let one = differ.run(&steps(&[SEED, PURGE])).expect("run");
    println!("{one}");
    one.assert_no_mismatch();
    copy_backup(&differ);
    let two = differ.run(&steps(&[RESTORE])).expect("run");
    println!("{two}");
    two.assert_no_mismatch();
}
