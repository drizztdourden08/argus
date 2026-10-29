//! Ports the routing rules of `coord.mjs`: help, the unknown verb, every verb and alias, the
//! project lookup, the error to exit code map, the hook that never breaks a session, and the
//! exit 120 of a verb that is not ported: nothing read from stdin, nothing written, no byte
//! printed, so the shim can run the Node verb with nothing done twice.

mod common;

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{self, Read};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use argus_coord::NOT_PORTED;
use argus_coord::cli::route::{Entry, dispatch, module_of};
use argus_coord::cli::{Call, CoordError, Invocation, Outcome, USAGE};
use common::{Home, board_with, coord};

/// Every verb and alias of `coord.mjs`, with `hook`, `dashboard` and `portfolio`.
const VERBS: [&str; 37] = [
    "hook",
    "dashboard",
    "thread",
    "agent",
    "item",
    "test",
    "build",
    "launcher",
    "ask",
    "decide",
    "notify",
    "message",
    "answer",
    "entry",
    "lock",
    "action",
    "todo",
    "log",
    "inbox",
    "board",
    "sync",
    "check",
    "tick",
    "dispatch",
    "purge",
    "restore",
    "trash",
    "portfolio",
    "lead",
    "ping",
    "wake-fault",
    "proc",
    "activity",
    "run",
    "worktree",
    "remote",
    "pr",
];

/// Counts the reads made of it, and holds JSON the verb must never see.
struct Watched(Arc<AtomicUsize>);

impl Read for Watched {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.fetch_add(1, Ordering::SeqCst);
        let json = br#"{"session_id":"s","cwd":"."}"#;
        buf[..json.len()].copy_from_slice(json);
        Ok(json.len())
    }
}

/// A digest of every path and every byte under a folder.
fn tree_hash(dir: &Path) -> u64 {
    let mut hasher = DefaultHasher::new();
    let mut stack = vec![dir.to_path_buf()];
    let mut files = Vec::new();
    while let Some(next) = stack.pop() {
        for entry in fs::read_dir(&next).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files.sort();
    for file in files {
        file.hash(&mut hasher);
        fs::read(&file).unwrap().hash(&mut hasher);
    }
    hasher.finish()
}

fn seeded_home() -> Home {
    let home = Home::new();
    home.write_board(
        "alpha",
        &board_with("alpha", &[("ui", "thread", "active", Some("sid-ui"), None)]),
    );
    home
}

#[test]
fn an_unported_verb_exits_120_before_reading_stdin_or_touching_anything() {
    let home = seeded_home();
    let cwd = home.folder("checkout");
    let before = tree_hash(home.path());
    for verb in VERBS {
        let reads = Arc::new(AtomicUsize::new(0));
        let (mut inv, out, err) = Invocation::captured(
            &[verb, "x", "--project", "alpha"],
            home.env(),
            cwd.clone(),
            "",
        );
        inv.stdin = Box::new(Watched(Arc::clone(&reads)));
        let code = dispatch(&mut inv, &[]);
        assert_eq!(code, NOT_PORTED, "{verb}");
        assert_eq!(code, 120);
        assert_eq!(out.text(), "", "{verb} wrote on stdout");
        assert_eq!(err.text(), "", "{verb} wrote on stderr");
        assert_eq!(reads.load(Ordering::SeqCst), 0, "{verb} read stdin");
    }
    assert_eq!(
        tree_hash(home.path()),
        before,
        "a file under the home changed"
    );
}

#[test]
fn an_unported_hook_with_json_on_stdin_leaves_it_for_node() {
    let home = seeded_home();
    let cwd = home.folder("checkout");
    let before = tree_hash(home.path());
    let reads = Arc::new(AtomicUsize::new(0));
    let (mut inv, out, err) = Invocation::captured(&["hook", "tool"], home.env(), cwd, "");
    inv.stdin = Box::new(Watched(Arc::clone(&reads)));
    assert_eq!(dispatch(&mut inv, &[]), 120);
    assert_eq!((out.text(), err.text()), (String::new(), String::new()));
    assert_eq!(reads.load(Ordering::SeqCst), 0);
    assert_eq!(tree_hash(home.path()), before);
}

#[test]
fn the_exit_120_does_not_depend_on_the_project() {
    let home = Home::new();
    let bare = home.folder("bare");
    // no .ai-config.json above: a ported verb would refuse, an unported one never looks
    let (mut inv, out, err) = Invocation::captured(&["item", "list"], home.env(), bare, "");
    assert_eq!(
        (dispatch(&mut inv, &[]), out.text(), err.text()),
        (120, String::new(), String::new())
    );
}

#[test]
fn help_with_no_verb_or_the_word_help_prints_the_usage_and_exits_0() {
    let home = Home::new();
    for argv in [&[][..], &["help"][..], &["--help"][..]] {
        let ran = coord(argv, home.env(), home.path());
        assert_eq!(ran.code, 0);
        assert_eq!(ran.out, format!("{USAGE}\n"));
        assert_eq!(ran.err, "");
    }
    assert!(USAGE.starts_with("Usage: coord <verb> ... [--project <key>] [--as <thread>]\n"));
    assert!(USAGE.ends_with("coord hook session-start|prompt|tool|guard|stop"));
    assert_eq!(USAGE.lines().count(), 90);
}

#[test]
fn an_unknown_verb_says_so_then_the_usage_on_stderr_and_exits_1() {
    let home = Home::new();
    let ran = coord(&["frobnicate", "x"], home.env(), home.path());
    assert_eq!(ran.code, 1);
    assert_eq!(ran.out, "");
    assert_eq!(
        ran.err,
        format!("Unknown verb \"frobnicate\".\n\n{USAGE}\n")
    );
}

#[test]
fn every_alias_names_the_module_that_serves_it() {
    for (verb, module) in [
        ("ask", "entry"),
        ("decide", "entry"),
        ("notify", "entry"),
        ("message", "entry"),
        ("answer", "entry"),
        ("entry", "entry"),
        ("todo", "action"),
        ("action", "action"),
        ("restore", "purge"),
        ("trash", "purge"),
        ("purge", "purge"),
        ("dispatch", "dispatch"),
        ("wake-fault", "wakefault"),
    ] {
        assert_eq!(module_of(verb), Some(module), "{verb}");
    }
    assert_eq!(module_of("help"), None);
    assert_eq!(module_of("nope"), None);
    for verb in VERBS {
        assert!(module_of(verb).is_some(), "{verb}");
    }
}

#[allow(clippy::unnecessary_wraps)]
fn echo(call: &mut Call<'_>) -> Outcome {
    let line = format!(
        "{} | {}",
        call.project.as_deref().unwrap_or("-"),
        call.positional.join(",")
    );
    call.inv.say(&[&line]);
    Ok(())
}

fn refuse(_call: &mut Call<'_>) -> Outcome {
    Err(CoordError::refuse("no way."))
}

fn code_3(call: &mut Call<'_>) -> Outcome {
    call.inv.warn("coord: down");
    Err(CoordError::Exit(3))
}

fn panics(_call: &mut Call<'_>) -> Outcome {
    panic!("a hook bug");
}

fn hook_error(call: &mut Call<'_>) -> Outcome {
    call.inv.say(&["half an answer"]);
    Err(CoordError::refuse("the board is broken"))
}

fn wake(_call: &mut Call<'_>) -> Outcome {
    Err(CoordError::Exit(2))
}

fn ported(table: &[Entry], argv: &[&str], home: &Home, cwd: &Path) -> common::Ran {
    let (mut inv, out, err) = Invocation::captured(argv, home.env(), cwd.to_path_buf(), "");
    let code = dispatch(&mut inv, table);
    common::Ran {
        code,
        out: out.text(),
        err: err.text(),
    }
}

#[test]
fn a_project_verb_gets_the_project_and_the_words_after_the_verb() {
    let home = Home::new();
    let ran = ported(
        &[("thread", echo)],
        &["thread", "register", "ui", "--project", "alpha"],
        &home,
        home.path(),
    );
    assert_eq!((ran.code, ran.out.as_str()), (0, "alpha | register,ui\n"));
}

#[test]
fn entry_and_purge_are_told_which_alias_was_typed() {
    let home = Home::new();
    let table: [Entry; 2] = [("entry", echo), ("purge", echo)];
    for (argv, want) in [
        (&["ask", "I-1", "why?"][..], "alpha | ask,I-1,why?\n"),
        (&["notify", "hello"][..], "alpha | notify,hello\n"),
        (&["restore", "--from", "b.json"][..], "alpha | restore\n"),
        (&["trash", "list"][..], "alpha | trash,list\n"),
        (&["purge", "thread", "x"][..], "alpha | purge,thread,x\n"),
    ] {
        let mut full = argv.to_vec();
        full.extend(["--project", "alpha"]);
        let ran = ported(&table, &full, &home, home.path());
        assert_eq!(ran.out, want, "{argv:?}");
    }
}

#[test]
fn an_alias_of_a_module_is_served_by_that_module() {
    let home = Home::new();
    let ran = ported(
        &[("action", echo)],
        &["todo", "add", "step", "--project", "p"],
        &home,
        home.path(),
    );
    assert_eq!(ran.out, "p | add,step\n");
}

#[test]
fn a_project_that_cannot_be_found_refuses_before_the_verb_runs() {
    let home = Home::new();
    let bare = home.folder("bare");
    let ran = ported(&[("thread", echo)], &["thread", "list"], &home, &bare);
    assert_eq!(ran.code, 1);
    assert_eq!(ran.out, "");
    assert_eq!(
        ran.err,
        "coord: cannot tell which project this is: no .ai-config.json above the cwd. Pass --project <key>.\n"
    );
}

#[test]
fn the_session_registry_finds_the_project_for_a_verb() {
    let home = seeded_home();
    let env = home.env_with("CLAUDE_CODE_SESSION_ID", "sid-ui");
    let (mut inv, out, _) =
        Invocation::captured(&["thread", "list"], env, home.path().to_path_buf(), "");
    assert_eq!(dispatch(&mut inv, &[("thread", echo)]), 0);
    assert_eq!(out.text(), "alpha | list\n");
}

#[test]
fn portfolio_dashboard_lead_and_the_server_verbs_need_no_project() {
    let home = Home::new();
    let bare = home.folder("bare");
    for verb in [
        "portfolio",
        "dashboard",
        "lead",
        "wake-fault",
        "proc",
        "activity",
    ] {
        let module = module_of(verb).unwrap();
        let ran = ported(&[(module, echo)], &[verb, "x"], &home, &bare);
        assert_eq!((ran.code, ran.out.as_str()), (0, "- | x\n"), "{verb}");
    }
}

#[test]
fn a_refusal_prints_coord_colon_message_on_stderr_and_exits_1() {
    let home = Home::new();
    let ran = ported(&[("proc", refuse)], &["proc", "list"], &home, home.path());
    assert_eq!((ran.code, ran.out.as_str()), (1, ""));
    assert_eq!(ran.err, "coord: no way.\n");
}

#[test]
fn a_verb_that_picked_its_own_code_ends_with_it_and_nothing_more() {
    let home = Home::new();
    let ran = ported(
        &[("run", code_3)],
        &["run", "start", "--project", "p"],
        &home,
        home.path(),
    );
    assert_eq!(
        (ran.code, ran.out.as_str(), ran.err.as_str()),
        (3, "", "coord: down\n")
    );
}

#[test]
fn a_hook_never_breaks_the_session_an_error_or_a_panic_is_exit_0() {
    let home = Home::new();
    let failed = ported(
        &[("hook", hook_error)],
        &["hook", "tool"],
        &home,
        home.path(),
    );
    assert_eq!(failed.code, 0);
    assert_eq!(failed.err, "", "no refusal text from a hook");
    let crashed = ported(&[("hook", panics)], &["hook", "tool"], &home, home.path());
    assert_eq!((crashed.code, crashed.err.as_str()), (0, ""));
}

#[test]
fn a_listener_that_heard_a_wake_still_exits_2() {
    let home = Home::new();
    let ran = ported(&[("hook", wake)], &["hook", "listen"], &home, home.path());
    assert_eq!(ran.code, 2);
}
