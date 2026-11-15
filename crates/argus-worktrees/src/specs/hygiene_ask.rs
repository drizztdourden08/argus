//! KEEP: K3, K4.
//! Worktree hygiene on the fake git, the part with work left (the owner's rule of 2026-11-06): an unattached worktree
//! that holds work asks its lead once; unanswered for 30 minutes it is reported to the Overlord once; the lead's or the
//! Overlord's approval writes a full backup, then removes it; an agent coming back stops all of this; only the Overlord
//! puts a worktree on the exception list, and it holds. Then the `no item` notice, which removes nothing.

// test-tag: standard

use crate::board::{Caller, ItemFact, WorktreeLink};
use crate::hygiene::{HygieneState, KeepInput};
use crate::linking::LinkSource;
use crate::rule::Rule;
use crate::specs::hygiene_kit::{
    DEBOUNCE, HOUR, REPORT, add_overlord, approve_as_lead, backup_files, has_extension, hygiene_on,
    input_by, kind, left_why, notes_of, state, step, wakes, world, world_with,
};
use crate::testkit::{Fixture, Harness, branches, exists_itself, put, rule_for, write};
use std::path::Path;

/// Agent 1's worktree with one uncommitted file.
fn dirty_world() -> (Fixture, Harness) {
    world(|fx| {
        put(&fx.path("agent-a1"), "wip.txt", "not committed\n");
    })
}

/// Agent 1's worktree on a detached HEAD with a commit no branch or ref keeps. Git refuses a second worktree in the
/// same folder, so its clean worktree goes first.
fn detached_world() -> (Fixture, Harness) {
    world(|fx| {
        fx.git(&["worktree", "remove", &fx.path("agent-a1")]);
        fx.add_detached_work("agent-a1");
    })
}

/// The lead is asked: the debounce over.
fn asked(h: &Harness) {
    assert_eq!(step(h, DEBOUNCE).told, 1);
}

fn keep_by(h: &Harness, path: &str, by: &str) -> KeepInput {
    KeepInput {
        project: "alpha".into(),
        caller: Caller::Thread(by.into()),
        by_project: Some("portfolio".into()),
        id: h.id_of(path),
        note: Some("the owner tests it tonight".into()),
        until_ms: None,
        asked_by: Some("lead".into()),
        clear: false,
    }
}

#[test]
fn work_left_on_an_unattached_worktree_asks_its_lead_once_with_both_commands() {
    let (fx, h) = dirty_world();
    let path = fx.path("agent-a1");
    assert_eq!(left_why(&h, "agent-a1"), "it has 1 uncommitted change");
    assert_eq!(step(&h, DEBOUNCE - 1).told, 0, "the debounce is not over");
    assert_eq!(step(&h, 1).told, 1);
    let id = h.id_of(&path);
    let note = notes_of(&h, "lead").pop().unwrap();
    assert!(
        note.contains(&path)
            && note.contains(&format!("coord worktree approve {id} --project alpha"))
            && note.contains("coord lead portfolio")
            && note.contains("30 minutes"),
        "{note}"
    );
    let told = wakes(&h, "worktree-approve");
    assert_eq!(told.len(), 1, "{told:?}");
    assert_eq!((told[0].0.as_str(), told[0].1.as_str()), ("alpha", "lead"));
    assert!(told[0].2.chars().count() <= 300);
    assert!(
        h.docs()["agent-a1"]
            .hygiene
            .clone()
            .unwrap()
            .asked_at
            .is_some()
    );
    assert_eq!(step(&h, DEBOUNCE).told, 0, "asked once");
    h.svc.requests_idle();
    assert!(exists_itself(&path), "nothing goes without the approval");
}

#[test]
fn commits_no_ref_keeps_and_a_lock_ask_too_and_the_lock_says_to_unlock_first() {
    let (_fx, h) = detached_world();
    assert_eq!(
        left_why(&h, "agent-a1"),
        "its HEAD has commits no branch or ref keeps"
    );
    let (fx, h) = world(|fx| {
        fx.git(&[
            "worktree",
            "lock",
            "--reason",
            "owner: hands off",
            &fx.path("agent-a1"),
        ]);
    });
    asked(&h);
    let note = notes_of(&h, "lead").pop().unwrap();
    assert!(
        note.contains(&format!(
            "unlock it first (git worktree unlock {})",
            fx.path("agent-a1")
        )),
        "{note}"
    );
}

#[test]
fn an_ask_unanswered_for_30_minutes_is_reported_to_the_overlord_once() {
    let (fx, h) = dirty_world();
    let path = fx.path("agent-a1");
    asked(&h);
    assert_eq!(
        step(&h, REPORT).told,
        0,
        "no Overlord to tell: it tries again"
    );
    add_overlord(&h);
    assert_eq!(step(&h, 1).told, 1);
    let id = h.id_of(&path);
    let note = notes_of(&h, "overlord").pop().unwrap();
    assert!(
        note.contains(&path)
            && note.contains("it has 1 uncommitted change")
            && note.contains("Backup first:")
            && note.contains("(lead lead)")
            && note.contains(&format!(
                "Delete: coord worktree approve {id} --project alpha"
            )),
        "{note}"
    );
    let told = wakes(&h, "worktree-report");
    assert_eq!(told.len(), 1, "{told:?}");
    assert_eq!(
        (told[0].0.as_str(), told[0].1.as_str()),
        ("portfolio", "overlord")
    );
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Reported));
    assert!(
        h.docs()["agent-a1"]
            .hygiene
            .clone()
            .unwrap()
            .reported_at
            .is_some()
    );
    assert_eq!(step(&h, REPORT).told, 0, "reported once");
    assert!(exists_itself(&path));
}

#[test]
fn the_overlords_delete_writes_a_full_backup_and_his_result_lands_on_his_board() {
    let (fx, h) = dirty_world();
    let path = fx.path("agent-a1");
    add_overlord(&h);
    asked(&h);
    assert_eq!(step(&h, REPORT).told, 1);
    let queued = h.svc.approve(&input_by(&h, &path, "overlord")).unwrap();
    assert!(queued.text.contains("full backup"), "{}", queued.text);
    h.svc.requests_idle();
    assert!(!exists_itself(&path));
    let files = backup_files(&h);
    assert!(has_extension(&files, "patch"), "{files:?}");
    let note = notes_of(&h, "overlord").pop().unwrap();
    assert!(
        note.starts_with("approved worktree removal (request R-1 done)"),
        "{note}"
    );
    assert!(
        note.contains(".patch") && note.contains("restore:"),
        "{note}"
    );
    assert_eq!(state(&h, "agent-a1"), None);
}

#[test]
fn the_leads_approval_writes_a_full_backup_and_removes_it_with_its_merged_branch() {
    let (fx, h) = dirty_world();
    let path = fx.path("agent-a1");
    asked(&h);
    let queued = h.svc.approve(&input_by(&h, &path, "lead")).unwrap();
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Removing));
    assert!(
        queued.text.contains("its branch goes when merged"),
        "{}",
        queued.text
    );
    h.svc.requests_idle();
    assert!(!exists_itself(&path));
    assert!(!branches(&fx.repo).contains(&"agent-a1".to_string()));
    let files = backup_files(&h);
    assert!(
        has_extension(&files, "patch") && files.iter().any(|f| f.ends_with(".untracked.txt")),
        "{files:?}"
    );
    let note = notes_of(&h, "lead").pop().unwrap();
    assert!(
        note.starts_with("approved worktree removal (request R-1 done)"),
        "{note}"
    );
    let request = &h.svc.request_list("alpha")[0];
    assert!(request.discard && request.branch, "{request:?}");
}

#[test]
fn commits_no_ref_keeps_go_to_a_bundle_on_approval() {
    let (fx, h) = detached_world();
    asked(&h);
    approve_as_lead(&h, &fx.path("agent-a1"));
    assert!(!exists_itself(&fx.path("agent-a1")));
    assert!(has_extension(&backup_files(&h), "bundle"));
}

#[test]
fn an_approval_is_refused_while_anything_is_attached_and_to_the_page_and_a_stranger() {
    let (fx, h) = world_with(&[("1", "lead", "working")], |fx| {
        put(&fx.path("agent-a1"), "wip.txt", "not committed\n");
    });
    let path = fx.path("agent-a1");
    let err = h.svc.approve(&input_by(&h, &path, "lead")).unwrap_err();
    assert!(
        err.message
            .contains("was not approved for removal: its agent A-1 is working"),
        "{}",
        err.message
    );
    let mut page = input_by(&h, &path, "lead");
    page.caller = Caller::Page;
    let err = h.svc.approve(&page).unwrap_err();
    assert!(
        err.message
            .contains("approve is a lead's or the Overlord's word"),
        "{}",
        err.message
    );
    h.board().agents[0].status = "done".into();
    h.scan();
    let err = h.svc.approve(&input_by(&h, &path, "other")).unwrap_err();
    assert!(
        err.message.contains("other may not remove it"),
        "{}",
        err.message
    );
    assert!(exists_itself(&path));
}

#[test]
fn a_backup_that_would_be_partial_refuses_even_the_approval() {
    let (fx, h) = world(|_| {});
    for i in 0..2001 {
        write(&fx.path("agent-a1"), &format!("many/f{i}.txt"), "x");
    }
    h.scan();
    asked(&h);
    approve_as_lead(&h, &fx.path("agent-a1"));
    assert!(exists_itself(&fx.path("agent-a1")), "nothing was removed");
    let note = notes_of(&h, "lead").pop().unwrap();
    assert!(
        note.contains("refused 1") && note.contains("the backup of"),
        "{note}"
    );
    assert_eq!(
        backup_files(&h).len(),
        0,
        "a partial backup is never written"
    );
    step(&h, 1);
    assert!(left_why(&h, "agent-a1").starts_with("Argus could not remove it: "));
    assert_eq!(step(&h, DEBOUNCE).told, 0, "the lead is not asked again");
}

#[test]
fn an_agent_back_on_it_stops_the_ask_and_a_later_detach_asks_again() {
    let (fx, h) = dirty_world();
    add_overlord(&h);
    asked(&h);
    h.board().agents[0].status = "working".into();
    step(&h, REPORT);
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Attached));
    assert!(
        notes_of(&h, "overlord").is_empty(),
        "never reported while attached"
    );
    let ended = h.clock_now() - 1;
    {
        let mut board = h.board();
        board.agents[0].status = "done".into();
        board.agents[0].ended_ms = Some(ended);
    }
    h.scan();
    assert!(left_why(&h, "agent-a1").contains("uncommitted"));
    assert_eq!(step(&h, DEBOUNCE).told, 1, "a new debounce, a new ask");
    assert_eq!(notes_of(&h, "lead").len(), 2);
    assert!(exists_itself(&fx.path("agent-a1")));
}

#[test]
fn a_lead_thread_named_like_the_coordinator_only_asks_without_the_portfolio_board() {
    let (fx, h) = world(|_| {});
    let path = fx.path("agent-a1");
    add_overlord(&h);
    for from in [Some("alpha"), None] {
        let mut input = keep_by(&h, &path, "overlord");
        input.by_project = from.map(Into::into);
        let kept = h.svc.keep(&input).unwrap();
        assert!(
            kept.text.starts_with("pending keep filed"),
            "{from:?}: {}",
            kept.text
        );
    }
    h.svc.hygiene_step("alpha");
    let (_, why) = state(&h, "agent-a1").unwrap();
    assert!(why.unwrap().starts_with("pending keep"));
    assert!(h.svc.keep(&keep_by(&h, &path, "overlord")).is_ok());
}

#[test]
fn only_the_overlord_keeps_and_his_exception_holds_until_he_clears_it() {
    let (fx, h) = world(|_| {});
    let path = fx.path("agent-a1");
    add_overlord(&h);
    let pending = h.svc.keep(&keep_by(&h, &path, "lead")).unwrap();
    assert!(
        pending.text.starts_with("pending keep filed"),
        "{}",
        pending.text
    );
    let mut page = keep_by(&h, &path, "lead");
    page.caller = Caller::Page;
    assert!(h.svc.keep(&page).is_err(), "the page does not keep either");
    let kept = h.svc.keep(&keep_by(&h, &path, "overlord")).unwrap();
    assert!(kept.text.contains("on the exception list"), "{}", kept.text);
    h.svc.hygiene_step("alpha");
    assert_eq!(
        state(&h, "agent-a1"),
        Some((
            HygieneState::Exception,
            Some("the owner tests it tonight".into())
        ))
    );
    put(&path, "wip.txt", "not committed\n");
    h.scan();
    assert_eq!(
        step(&h, DEBOUNCE + REPORT).told,
        0,
        "never asked, never reported"
    );
    h.svc.requests_idle();
    assert!(exists_itself(&path) && h.svc.request_list("alpha").is_empty());
    let err = h.svc.approve(&input_by(&h, &path, "lead")).unwrap_err();
    assert!(err.message.contains("exception list"), "{}", err.message);
    let mut clear = keep_by(&h, &path, "overlord");
    clear.clear = true;
    h.svc.keep(&clear).unwrap();
    h.svc.hygiene_step("alpha");
    assert!(left_why(&h, "agent-a1").contains("uncommitted"));
}

#[test]
fn an_exception_ends_at_its_until() {
    let (fx, h) = world(|_| {});
    let path = fx.path("agent-a1");
    add_overlord(&h);
    let mut input = keep_by(&h, &path, "overlord");
    input.until_ms = Some(h.clock_now() + HOUR);
    h.svc.keep(&input).unwrap();
    step(&h, HOUR - 1);
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Exception));
    assert!(
        h.docs()["agent-a1"]
            .hygiene
            .clone()
            .unwrap()
            .until
            .is_some()
    );
    step(&h, 1);
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Removing));
    let mut past = keep_by(&h, &path, "overlord");
    past.until_ms = Some(h.clock_now());
    assert!(
        h.svc
            .keep(&past)
            .unwrap_err()
            .message
            .contains("--until must be in the future")
    );
}

fn linked(item: &str) -> WorktreeLink {
    WorktreeLink {
        item: item.into(),
        by: Some("lead".into()),
        at_ms: None,
    }
}

#[test]
fn a_worktree_with_no_item_an_hour_after_it_was_first_seen_is_flagged_and_told_once_and_the_flag_removes_nothing()
 {
    let fx = Fixture::fake();
    fx.add("agent-a9");
    put(&fx.path("agent-a9"), "wip.txt", "not committed\n");
    let h = Harness::new(
        &fx.root,
        vec![rule_for(&fx).filtered(&["agent-*"], &[])],
        hygiene_on,
    );
    h.board().items.insert(
        "I-1".into(),
        ItemFact {
            id: "I-1".into(),
            lane: "doing".into(),
            trashed: false,
            text: String::new(),
            thread: None,
        },
    );
    h.scan();
    h.svc.start();
    assert!(!h.docs()["agent-a9"].link.unlinked);
    assert_eq!(step(&h, HOUR - 1).noticed, 0);
    assert!(!h.docs()["agent-a9"].link.unlinked);
    assert_eq!(step(&h, 1).noticed, 1);
    assert!(h.docs()["agent-a9"].link.unlinked);
    let told = wakes(&h, "worktree-unlinked");
    assert_eq!(told.len(), 1, "{told:?}");
    assert_eq!(told[0].1, "worker");
    assert!(told[0].2.contains(&fx.path("agent-a9")), "{}", told[0].2);
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
    let h = Harness::new(&fx.root, vec![main_rule], hygiene_on);
    h.scan();
    h.svc.start();
    step(&h, HOUR);
    assert_eq!(step(&h, HOUR).noticed, 0);
    assert!(!h.docs().get("proj").is_some_and(|d| d.link.unlinked));
    assert_eq!(
        wakes(&h, "worktree-unlinked"),
        Vec::<(String, String, String)>::new()
    );
    assert!(Path::new(&fx.repo).exists());
}
