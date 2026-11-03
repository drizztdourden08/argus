//! KEEP: K3, K4.
//! Worktrees close with their item (owner, I-78; decision D-42): a worktree is linked to an item; when the item is done,
//! dropped or trashed it is `closing`, and after a grace a reopen cancels it goes through the very same removal, its
//! backup first. What still stops it (a running agent, a process, a partial backup), the keep that does not hold it, the
//! switch that turns it off, the `no item` flag and the note the lead gets.

// test-tag: standard

use crate::board::{AgentFact, Caller, ItemFact, WorktreeLink};
use crate::hygiene::{HygieneState, Stepped};
use crate::linking::LinkSource;
use crate::request::RemoveInput;
use crate::requests::Mode;
use crate::rule::Rule;
use crate::sync::locked;
use crate::testkit::{Fixture, Harness, exists_itself, put, rule_for, write};
use std::path::Path;

const CLOSE_GRACE: i64 = 120_000;
const HOUR: i64 = 3_600_000;

fn item(lane: &str) -> ItemFact {
    ItemFact {
        id: "I-1".into(),
        lane: lane.into(),
        trashed: false,
        text: String::new(),
    }
}

fn linked(item: &str) -> WorktreeLink {
    WorktreeLink {
        item: item.into(),
        by: Some("lead".into()),
        at_ms: None,
    }
}

/// A service over a fixture with the worktrees `names` (`agent-a1`, `agent-a2` are linked to the open item I-1,
/// `agent-a9` is not), the hygiene on, and the request worker started.
fn world_with(
    names: &[&str],
    prep: impl FnOnce(&Fixture),
    options: impl FnOnce(&mut crate::service::Options),
) -> (Fixture, Harness) {
    let fx = Fixture::fake();
    for name in names {
        fx.add(name);
    }
    prep(&fx);
    let rule = rule_for(&fx).filtered(&["agent-*"], &[]);
    let h = Harness::new(&fx.root, vec![rule], |o| {
        o.release = true;
        o.item_close_grace_ms = CLOSE_GRACE;
        options(o);
    });
    {
        let mut board = h.board();
        board.items.insert("I-1".into(), item("doing"));
        for name in ["agent-a1", "agent-a2"] {
            board
                .worktree_links
                .insert(h.id_of(&fx.path(name)), linked("I-1"));
        }
    }
    h.scan();
    h.svc.start();
    (fx, h)
}

fn world(names: &[&str]) -> (Fixture, Harness) {
    world_with(names, |_| {}, |_| {})
}

fn set_lane(h: &Harness, lane: &str) {
    h.board().items.get_mut("I-1").unwrap().lane = lane.into();
}

fn state(h: &Harness, name: &str) -> Option<HygieneState> {
    h.docs()
        .get(name)
        .and_then(|d| d.hygiene.as_ref())
        .map(|g| g.state)
}

fn review_why(h: &Harness, name: &str) -> String {
    match h.docs().get(name).and_then(|d| d.hygiene.clone()) {
        Some(g) if g.state == HygieneState::Review => g.why.unwrap_or_default(),
        other => panic!("{name} should be in review, it is {other:?}"),
    }
}

fn step(h: &Harness, ms: i64) -> Stepped {
    h.clock.advance(ms);
    h.svc.hygiene_step("alpha")
}

/// A holder keeps the worktree of the closed item in use through twice the grace.
fn holds_through_the_grace(h: &Harness, fx: &Fixture) {
    h.scan();
    assert_eq!(state(h, "agent-a1"), Some(HygieneState::InUse));
    step(h, CLOSE_GRACE * 2);
    h.svc.requests_idle();
    assert!(
        exists_itself(&fx.path("agent-a1")),
        "a holder keeps it in use"
    );
}

/// The holder is gone: the worktree is closing, and it goes after the grace.
fn goes_after_the_grace(h: &Harness, fx: &Fixture) {
    h.scan();
    assert_eq!(state(h, "agent-a1"), Some(HygieneState::Closing));
    step(h, CLOSE_GRACE);
    h.svc.requests_idle();
    assert!(!exists_itself(&fx.path("agent-a1")));
}

/// The item closes, the grace passes, the request runs.
fn close_and_wait(h: &Harness) {
    set_lane(h, "done");
    h.svc.hygiene_step("alpha");
    step(h, CLOSE_GRACE);
    h.svc.requests_idle();
}

fn inbox(h: &Harness) -> Vec<(String, String)> {
    locked(&h.store.inbox).clone()
}

fn wakes(h: &Harness, reason: &str) -> Vec<(String, String)> {
    locked(&h.notes.wakes)
        .iter()
        .filter(|w| w.reason == reason)
        .map(|w| (w.thread.clone(), w.text.clone()))
        .collect()
}

fn backup_files(h: &Harness) -> Vec<String> {
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
fn has_extension(files: &[String], wanted: &str) -> bool {
    files.iter().any(|f| {
        Path::new(f)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case(wanted))
    })
}

#[test]
fn a_clean_worktree_of_a_closed_item_goes_after_the_grace_with_no_backup_and_the_lead_gets_one_note()
 {
    let (fx, h) = world(&["agent-a1"]);
    assert_eq!(state(&h, "agent-a1"), Some(HygieneState::Released));
    set_lane(&h, "done");
    h.svc.hygiene_step("alpha");
    assert_eq!(state(&h, "agent-a1"), Some(HygieneState::Closing));
    let hygiene = h.docs()["agent-a1"].hygiene.clone().unwrap();
    assert_eq!(hygiene.item.as_deref(), Some("I-1"));
    assert!(hygiene.goes_at.is_some());
    step(&h, CLOSE_GRACE - 1);
    h.svc.requests_idle();
    assert!(
        exists_itself(&fx.path("agent-a1")) && h.svc.request_list("alpha").is_empty(),
        "the grace is not over"
    );
    step(&h, 1);
    h.svc.requests_idle();
    assert!(!exists_itself(&fx.path("agent-a1")));
    assert_eq!(backup_files(&h).len(), 0, "clean and kept: nothing is lost");
    let sent = h.svc.request_list("alpha");
    assert_eq!(sent.len(), 1);
    assert_eq!(
        (sent[0].mode, sent[0].removed(), sent[0].refused()),
        (Mode::ItemClosed, 1, 0)
    );
    let notes = inbox(&h);
    assert_eq!(notes.len(), 1, "{notes:?}");
    assert_eq!(notes[0].0, "worker");
    assert!(
        notes[0]
            .1
            .starts_with("worktrees removed with their closed items (request R-1 done): removed 1: I-1: removed worktree"),
        "{}",
        notes[0].1
    );
    assert_eq!(state(&h, "agent-a1"), None);
}

#[test]
fn done_dropped_and_trashed_all_close_a_worktree_and_an_open_item_does_not() {
    for (lane, trashed, goes) in [
        ("done", false, true),
        ("dropped", false, true),
        ("todo", true, true),
        ("review", false, false),
    ] {
        let (fx, h) = world(&["agent-a1"]);
        h.board().items.get_mut("I-1").unwrap().trashed = trashed;
        h.board().items.get_mut("I-1").unwrap().lane = lane.into();
        h.svc.hygiene_step("alpha");
        step(&h, CLOSE_GRACE);
        h.svc.requests_idle();
        assert_eq!(
            !exists_itself(&fx.path("agent-a1")),
            goes,
            "{lane} trashed {trashed}"
        );
    }
}

#[test]
fn uncommitted_work_is_written_to_a_patch_first_and_the_note_names_it_and_how_to_restore_it() {
    let (fx, h) = world(&["agent-a1"]);
    put(&fx.path("agent-a1"), "wip.txt", "not committed\n");
    h.scan();
    close_and_wait(&h);
    assert!(!exists_itself(&fx.path("agent-a1")));
    let files = backup_files(&h);
    assert!(has_extension(&files, "patch"), "{files:?}");
    assert!(
        files.iter().any(|f| f.ends_with(".untracked.txt")),
        "{files:?}"
    );
    let note = inbox(&h).pop().unwrap().1;
    assert!(
        note.contains(".patch") && note.contains("restore:") && note.contains("apply"),
        "{note}"
    );
}

#[test]
fn commits_no_ref_keeps_are_written_to_a_bundle_first() {
    let (fx, h) = world_with(&[], |fx| fx.add_detached_work("agent-a1"), |_| {});
    assert_eq!(
        review_why(&h, "agent-a1"),
        "its HEAD has commits no branch or ref keeps"
    );
    close_and_wait(&h);
    assert!(!exists_itself(&fx.path("agent-a1")));
    let files = backup_files(&h);
    assert!(has_extension(&files, "bundle"), "{files:?}");
    let note = inbox(&h).pop().unwrap().1;
    assert!(
        note.contains(".bundle") && note.contains("restore:"),
        "{note}"
    );
}

#[test]
fn a_backup_that_would_be_partial_refuses_and_the_worktree_stays_in_review_and_the_lead_is_told_once()
 {
    let (fx, h) = world(&["agent-a1"]);
    for i in 0..2001 {
        write(&fx.path("agent-a1"), &format!("many/f{i}.txt"), "x");
    }
    h.scan();
    close_and_wait(&h);
    assert!(exists_itself(&fx.path("agent-a1")), "nothing was removed");
    let note = inbox(&h).pop().unwrap().1;
    assert!(
        note.contains("refused 1") && note.contains("the backup of"),
        "{note}"
    );
    let told = step(&h, 1);
    let why = review_why(&h, "agent-a1");
    assert!(
        why.starts_with("its item I-1 closed, but Argus could not remove it: the backup of")
            && why.contains("only listed in the backup, not copied"),
        "{why}"
    );
    assert_eq!(told.noticed, 1);
    let stale = wakes(&h, "worktree-stale");
    assert_eq!(stale.len(), 1, "{stale:?}");
    assert_eq!(stale[0].0, "worker");
    assert!(
        stale[0].1.contains("of closed items stay"),
        "{}",
        stale[0].1
    );
    assert_eq!(step(&h, 1).noticed, 0, "told once");
    assert_eq!(
        backup_files(&h).len(),
        0,
        "a partial backup is never written"
    );
}

#[test]
fn a_running_agent_holds_it_and_it_goes_when_the_agent_ends() {
    let (fx, h) = world(&["agent-a1"]);
    let ended = h.clock_now() - 1000;
    h.board().agents = vec![AgentFact {
        id: "A-1".into(),
        thread: Some("lead".into()),
        status: "working".into(),
        claude_id: Some("a1".into()),
        item: Some("I-1".into()),
        ..AgentFact::default()
    }];
    set_lane(&h, "done");
    holds_through_the_grace(&h, &fx);
    {
        let mut board = h.board();
        board.agents[0].status = "done".into();
        board.agents[0].ended_ms = Some(ended);
    }
    goes_after_the_grace(&h, &fx);
    assert_eq!(
        inbox(&h).pop().unwrap().0,
        "lead",
        "the agent's thread is told"
    );
}

#[test]
fn a_worktree_doc_names_the_agent_that_owns_it_a_running_one_before_an_ended_one() {
    let (_fx, h) = world(&["agent-a1", "agent-a2"]);
    assert_eq!(h.docs()["agent-a1"].link.agent, None);
    let record = |id: &str, claude: &str, status: &str, ended_ms: Option<i64>| AgentFact {
        id: id.into(),
        thread: Some("lead".into()),
        status: status.into(),
        claude_id: Some(claude.into()),
        ended_ms,
        ..AgentFact::default()
    };
    h.board().agents = vec![
        record("A-1", "a1", "working", None),
        record("A-2", "a1", "done", Some(h.clock_now() - 1000)),
        record("A-3", "a2", "done", Some(h.clock_now() - 1000)),
    ];
    h.scan();
    assert_eq!(h.docs()["agent-a1"].link.agent.as_deref(), Some("A-1"));
    assert_eq!(h.docs()["agent-a2"].link.agent.as_deref(), Some("A-3"));
}

#[test]
fn a_process_in_it_holds_it_and_it_goes_when_the_process_ends() {
    let (fx, h) = world(&["agent-a1"]);
    set_lane(&h, "done");
    h.clock.advance(10_001);
    h.run_node_in(&fx.path("agent-a1"));
    holds_through_the_grace(&h, &fx);
    locked(&h.procs.0).clear();
    h.clock.advance(10_001);
    goes_after_the_grace(&h, &fx);
}

#[test]
fn a_reopen_inside_the_grace_cancels_it_and_so_does_a_link_to_another_open_item() {
    let (fx, h) = world(&["agent-a1", "agent-a2"]);
    h.board().items.insert(
        "I-2".into(),
        ItemFact {
            id: "I-2".into(),
            ..item("doing")
        },
    );
    set_lane(&h, "done");
    h.svc.hygiene_step("alpha");
    step(&h, CLOSE_GRACE - 1);
    set_lane(&h, "doing");
    h.board()
        .worktree_links
        .insert(h.id_of(&fx.path("agent-a2")), linked("I-2"));
    step(&h, 1);
    h.svc.requests_idle();
    assert_eq!(h.svc.request_list("alpha").len(), 0);
    assert!(exists_itself(&fx.path("agent-a1")) && exists_itself(&fx.path("agent-a2")));
    assert_eq!(state(&h, "agent-a1"), Some(HygieneState::Released));
    set_lane(&h, "done");
    step(&h, 1);
    assert_eq!(state(&h, "agent-a1"), Some(HygieneState::Closing));
    assert_eq!(
        state(&h, "agent-a2"),
        Some(HygieneState::Released),
        "linked to I-2, which is open"
    );
}

#[test]
fn a_keep_does_not_hold_the_worktree_of_a_closed_item() {
    let (fx, h) = world(&["agent-a1"]);
    h.keep_for_a_day(&fx.path("agent-a1"));
    h.svc.hygiene_step("alpha");
    assert_eq!(state(&h, "agent-a1"), Some(HygieneState::Kept));
    close_and_wait(&h);
    assert!(!exists_itself(&fx.path("agent-a1")));
}

#[test]
fn a_lock_that_is_not_brocks_stops_it_with_the_item_in_the_reason() {
    let (fx, h) = world(&["agent-a1"]);
    fx.git(&[
        "worktree",
        "lock",
        "--reason",
        "owner: hands off",
        &fx.path("agent-a1"),
    ]);
    h.scan();
    set_lane(&h, "done");
    h.svc.hygiene_step("alpha");
    assert_eq!(
        review_why(&h, "agent-a1"),
        "its item I-1 closed, but it is locked (owner: hands off)"
    );
    step(&h, CLOSE_GRACE * 2);
    h.svc.requests_idle();
    assert!(exists_itself(&fx.path("agent-a1")));
}

#[test]
fn with_the_switch_off_nothing_closes_with_its_item_and_a_removal_asks_the_owner_word_for_word() {
    let (fx, h) = world_with(&["agent-a1"], |_| {}, |o| o.close_with_item = false);
    put(&fx.path("agent-a1"), "wip.txt", "not committed\n");
    h.scan();
    close_and_wait(&h);
    assert!(exists_itself(&fx.path("agent-a1")));
    assert_eq!(
        review_why(&h, "agent-a1"),
        "it has 1 uncommitted change",
        "today's rule, with no word of the item"
    );
    let mut input = RemoveInput::new(
        "alpha",
        &h.id_of(&fx.path("agent-a1")),
        Caller::ItemClosed { item: "I-1".into() },
    );
    input.discard = true;
    let err = h.svc.remove(&input).unwrap_err();
    assert_eq!(
        err.message,
        format!(
            "discarding 1 uncommitted change in {} needs the owner: no owner entry was given (--owner-entry <id>: the owner's answer or note that asks for it)",
            fx.path("agent-a1")
        )
    );
}

#[test]
fn the_item_closed_caller_discards_only_for_an_item_that_is_closed_on_the_board() {
    let (fx, h) = world(&["agent-a1"]);
    put(&fx.path("agent-a1"), "wip.txt", "not committed\n");
    h.scan();
    let ask = |item: &str| {
        let mut input = RemoveInput::new(
            "alpha",
            &h.id_of(&fx.path("agent-a1")),
            Caller::ItemClosed { item: item.into() },
        );
        input.discard = true;
        h.svc.remove(&input)
    };
    let open = ask("I-1").unwrap_err();
    assert!(
        open.message
            .ends_with("needs the owner: item I-1 is not closed"),
        "{}",
        open.message
    );
    let unknown = ask("I-7").unwrap_err();
    assert!(
        unknown
            .message
            .contains("there is no item I-7 on this board"),
        "{}",
        unknown.message
    );
    assert!(exists_itself(&fx.path("agent-a1")));
    set_lane(&h, "done");
    let out = ask("I-1").unwrap();
    assert!(out.removed && out.backup.is_some(), "{}", out.text);
    assert!(!exists_itself(&fx.path("agent-a1")));
}

#[test]
fn a_worktree_with_no_item_an_hour_after_it_was_first_seen_is_flagged_and_told_once_and_the_flag_removes_nothing()
 {
    let (fx, h) = world_with(
        &["agent-a9"],
        |fx| {
            put(&fx.path("agent-a9"), "wip.txt", "not committed\n");
        },
        |_| {},
    );
    assert!(!h.docs()["agent-a9"].link.unlinked);
    let early = step(&h, HOUR - 1);
    assert_eq!(early.noticed, 0);
    assert!(!h.docs()["agent-a9"].link.unlinked);
    let flagged = step(&h, 1);
    assert_eq!(flagged.noticed, 1);
    assert!(h.docs()["agent-a9"].link.unlinked);
    let told = wakes(&h, "worktree-unlinked");
    assert_eq!(told.len(), 1, "{told:?}");
    assert_eq!(told[0].0, "worker");
    assert!(told[0].1.contains(&fx.path("agent-a9")), "{}", told[0].1);
    assert_eq!(step(&h, HOUR).noticed, 0, "told once");
    h.svc.requests_idle();
    assert!(
        exists_itself(&fx.path("agent-a9")),
        "having no item is not a reason to remove"
    );
    h.board()
        .worktree_links
        .insert(h.id_of(&fx.path("agent-a9")), linked("I-1"));
    step(&h, 1);
    let row = &h.docs()["agent-a9"];
    assert_eq!(
        (
            row.link.unlinked,
            row.link.item.as_deref(),
            row.link.link_source
        ),
        (false, Some("I-1"), Some(LinkSource::Manual))
    );
}

#[test]
fn a_main_checkout_is_never_flagged_for_having_no_item() {
    let fx = Fixture::fake();
    let main_rule = Rule::folder("W-2", &fx.root, "worker").filtered(&["proj"], &[]);
    let h = Harness::new(&fx.root, vec![main_rule], |o| o.release = true);
    h.scan();
    h.svc.start();
    step(&h, HOUR);
    let flagged = step(&h, HOUR);
    assert_eq!(flagged.noticed, 0);
    assert!(!h.docs().get("proj").is_some_and(|d| d.link.unlinked));
    assert_eq!(wakes(&h, "worktree-unlinked").len(), 0);
    assert!(Path::new(&fx.repo).exists());
}
