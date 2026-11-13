//! Helpers shared by the hygiene specs: a world of agents and their worktrees with the hygiene on, the state of a row,
//! one step on the clock, the notes, the wakes, the backups and the Overlord's board.

use crate::board::{AgentFact, BoardFacts, Caller, ThreadInfo};
use crate::hygiene::{HygieneState, Stepped};
use crate::ports::WorktreeStore;
use crate::request::RemoveInput;
use crate::rule::Rule;
use crate::service::Options;
use crate::sync::locked;
use crate::testkit::{Fixture, Harness, rule_for};
use std::path::Path;

/// How long a worktree stays unattached before the hygiene acts.
pub const DEBOUNCE: i64 = 60_000;
/// How long an ask stays unanswered before the Overlord is told.
pub const REPORT: i64 = 30 * 60_000;
pub const HOUR: i64 = 3_600_000;

pub fn agent(n: &str, thread: &str, status: &str, ended_ms: i64) -> AgentFact {
    AgentFact {
        id: format!("A-{n}"),
        thread: Some(thread.to_string()),
        status: status.to_string(),
        worktree: None,
        claude_id: Some(format!("a{n}")),
        ended_ms: Some(ended_ms),
        item: None,
    }
}

/// The rule over the agents' own folders and `named-work`: the other worktrees of the fixture are not listed.
pub fn agents_rule(fx: &Fixture) -> Rule {
    rule_for(fx).filtered(&["agent-*", "named-work"], &[])
}

/// The hygiene on, with the debounce and the report delay of these specs.
pub fn hygiene_on(o: &mut Options) {
    o.hygiene = true;
    o.detach_debounce_ms = DEBOUNCE;
    o.report_after_ms = REPORT;
}

/// A harness over the fixture with the hygiene on.
pub fn hygienic(fx: &Fixture, rule: Rule) -> Harness {
    Harness::new(&fx.root, vec![rule], hygiene_on)
}

/// A service over a fixture with one clean worktree `agent-a<n>` per agent (at the tip of main: merged), the agents on
/// the board, one scan, and the request worker started.
pub fn world_with(
    agents: &[(&str, &str, &str)],
    prep: impl FnOnce(&Fixture),
) -> (Fixture, Harness) {
    let fx = Fixture::fake_bare();
    for (n, _, _) in agents {
        fx.add(&format!("agent-a{n}"));
    }
    prep(&fx);
    let h = hygienic(&fx, agents_rule(&fx));
    let ended = h.clock_now() - 1000;
    h.board().agents = agents
        .iter()
        .map(|(n, thread, status)| agent(n, thread, status, ended))
        .collect();
    h.scan();
    h.svc.start();
    (fx, h)
}

/// Agent 1 of the lead, done, and its worktree.
pub fn world(prep: impl FnOnce(&Fixture)) -> (Fixture, Harness) {
    world_with(&[("1", "lead", "done")], prep)
}

pub fn state(h: &Harness, name: &str) -> Option<(HygieneState, Option<String>)> {
    h.docs()
        .get(name)
        .and_then(|d| d.hygiene.clone())
        .map(|g| (g.state, g.why))
}

pub fn kind(h: &Harness, name: &str) -> Option<HygieneState> {
    state(h, name).map(|s| s.0)
}

/// What is left on a worktree that waits for its lead; fails the test when it does not.
pub fn left_why(h: &Harness, name: &str) -> String {
    match state(h, name) {
        Some((HygieneState::AwaitingLead, Some(why))) => why,
        other => panic!("{name} should await its lead, it is {other:?}"),
    }
}

/// Why a worktree is attached; fails the test when it is not.
pub fn attached_why(h: &Harness, name: &str) -> String {
    match state(h, name) {
        Some((HygieneState::Attached, Some(why))) => why,
        other => panic!("{name} should be attached, it is {other:?}"),
    }
}

/// The clock moves on, then one step.
pub fn step(h: &Harness, ms: i64) -> Stepped {
    h.clock.advance(ms);
    h.svc.hygiene_step("alpha")
}

/// The debounce over and the requests done. Returns whether the folder is gone.
pub fn through_debounce(h: &Harness, path: &str) -> bool {
    step(h, DEBOUNCE);
    h.svc.requests_idle();
    !crate::testkit::exists_itself(path)
}

pub fn inbox(h: &Harness) -> Vec<(String, String)> {
    locked(&h.store.inbox).clone()
}

/// The notes of one thread.
pub fn notes_of(h: &Harness, thread: &str) -> Vec<String> {
    inbox(h)
        .into_iter()
        .filter(|(t, _)| t == thread)
        .map(|(_, text)| text)
        .collect()
}

/// The wakes of one reason: scope, thread and text.
pub fn wakes(h: &Harness, reason: &str) -> Vec<(String, String, String)> {
    locked(&h.notes.wakes)
        .iter()
        .filter(|w| w.reason == reason)
        .map(|w| (w.scope.clone(), w.thread.clone(), w.text.clone()))
        .collect()
}

/// The Overlord, `overlord`, live on the `portfolio` board.
pub fn add_overlord(h: &Harness) {
    let mut board = BoardFacts::default();
    board.threads.insert(
        "overlord".to_string(),
        ThreadInfo {
            id: "overlord".to_string(),
            coordinator: true,
            live: true,
        },
    );
    locked(&h.store.others).insert(h.store.overlord_board(), board);
}

/// The input of `by` about the worktree at `path`.
pub fn input_by(h: &Harness, path: &str, by: &str) -> RemoveInput {
    RemoveInput::new("alpha", &h.id_of(path), Caller::Thread(by.to_string()))
}

pub fn backup_files(h: &Harness) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(&h.backups)
        .map(|dir| {
            dir.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

/// Whether any backup file carries the extension, compared without case.
pub fn has_extension(files: &[String], wanted: &str) -> bool {
    files.iter().any(|f| {
        Path::new(f)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case(wanted))
    })
}
