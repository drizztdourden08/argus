//! KEEP: K3, K4.
//! Worktree hygiene on the fake git, the approval of several worktrees at once (I-129): the ids that pass become ONE
//! request, the ones that fail are reported by id, and a merged branch goes with its approved worktree while an
//! unmerged one stays (I-133).

// test-tag: standard

use crate::board::Caller;
use crate::hygiene::{ApproveInput, HygieneState};
use crate::specs::hygiene_kit::{DEBOUNCE, backup_files, input_by, kind, step, world, world_with};
use crate::testkit::{Fixture, Harness, branches, commit, exists_itself, put};

/// The lead is asked: the debounce over.
fn asked(h: &Harness) {
    assert_eq!(step(h, DEBOUNCE).told, 1);
}

/// Agents 1 and 2 done, agent 3 still working, each on a detached HEAD with a commit no ref keeps.
fn three_detached() -> (Fixture, Harness) {
    let agents = [
        ("1", "lead", "done"),
        ("2", "lead", "done"),
        ("3", "lead", "working"),
    ];
    world_with(&agents, |fx| {
        for name in ["agent-a1", "agent-a2", "agent-a3"] {
            fx.git(&["worktree", "remove", &fx.path(name)]);
            fx.add_detached_work(name);
        }
    })
}

fn approve_ids(h: &Harness, paths: &[String], by: &str) -> ApproveInput {
    ApproveInput {
        project: "alpha".into(),
        ids: paths.iter().map(|p| h.id_of(p)).collect(),
        caller: Caller::Thread(by.into()),
        reason: None,
    }
}

#[test]
fn several_ids_become_one_request_that_removes_each_with_its_backup() {
    let (fx, h) = three_detached();
    let paths = [fx.path("agent-a1"), fx.path("agent-a2")];
    asked(&h);
    let done = h
        .svc
        .approve_many(&approve_ids(&h, &paths, "lead"))
        .unwrap();
    assert!(done.refused.is_empty(), "{:?}", done.refused);
    let queued = done.queued.unwrap();
    assert!(
        queued.text.contains("each of 2 worktrees"),
        "{}",
        queued.text
    );
    assert_eq!(h.svc.request_list("alpha").len(), 1, "one request, not two");
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Removing));
    assert_eq!(kind(&h, "agent-a2"), Some(HygieneState::Removing));
    h.svc.requests_idle();
    assert!(!exists_itself(&paths[0]) && !exists_itself(&paths[1]));
    let bundles = backup_files(&h)
        .iter()
        .filter(|f| f.ends_with(".bundle"))
        .count();
    assert_eq!(bundles, 2, "{:?}", backup_files(&h));
    let request = &h.svc.request_list("alpha")[0];
    assert_eq!(
        (request.targets.len(), request.removed()),
        (2, 2),
        "{request:?}"
    );
}

#[test]
fn a_mix_of_good_and_refused_ids_removes_the_good_ones_and_reports_the_rest_by_id() {
    let (fx, h) = three_detached();
    let (good, attached) = (fx.path("agent-a1"), fx.path("agent-a3"));
    asked(&h);
    let mut input = approve_ids(&h, &[good.clone(), attached.clone()], "lead");
    input.ids.push("0000000000".into());
    let done = h.svc.approve_many(&input).unwrap();
    assert!(done.queued.is_some());
    assert_eq!(done.refused.len(), 2, "{:?}", done.refused);
    assert_eq!(done.refused[0].id, h.id_of(&attached));
    assert!(
        done.refused[0].text.contains("its agent A-3 is working"),
        "{}",
        done.refused[0].text
    );
    assert_eq!(done.refused[1].code, "not_found");
    h.svc.requests_idle();
    assert!(!exists_itself(&good));
    assert!(exists_itself(&attached), "the attached one stays");
    assert_eq!(h.svc.request_list("alpha")[0].targets.len(), 1);
}

#[test]
fn when_every_id_is_refused_no_request_is_filed() {
    let (fx, h) = three_detached();
    let done = h
        .svc
        .approve_many(&approve_ids(&h, &[fx.path("agent-a3")], "lead"))
        .unwrap();
    assert!(done.queued.is_none());
    assert_eq!(done.refused.len(), 1);
    assert_eq!(h.svc.request_list("alpha").len(), 0);
}

#[test]
fn the_single_id_form_still_files_its_own_request() {
    let (fx, h) = three_detached();
    let path = fx.path("agent-a1");
    asked(&h);
    let queued = h.svc.approve(&input_by(&h, &path, "lead")).unwrap();
    assert!(
        queued.text.contains("its branch goes when merged"),
        "{}",
        queued.text
    );
    h.svc.requests_idle();
    assert!(!exists_itself(&path));
    assert!(exists_itself(&fx.path("agent-a2")));
}

#[test]
fn the_page_and_an_empty_list_cannot_approve_several() {
    let (fx, h) = three_detached();
    let mut input = approve_ids(&h, &[fx.path("agent-a1")], "lead");
    input.caller = Caller::Page;
    assert!(h.svc.approve_many(&input).is_err());
    input.caller = Caller::Thread("lead".into());
    input.ids.clear();
    assert!(h.svc.approve_many(&input).is_err());
}

#[test]
fn an_approved_removal_deletes_a_merged_branch() {
    let (fx, h) = world(|fx| {
        put(&fx.path("agent-a1"), "wip.txt", "not committed\n");
    });
    asked(&h);
    h.svc
        .approve(&input_by(&h, &fx.path("agent-a1"), "lead"))
        .unwrap();
    h.svc.requests_idle();
    assert!(!exists_itself(&fx.path("agent-a1")));
    assert!(!branches(&fx.repo).contains(&"agent-a1".to_string()));
    assert!(h.svc.request_list("alpha")[0].branch);
}

#[test]
fn an_approved_removal_keeps_an_unmerged_branch() {
    let (fx, h) = world(|fx| {
        let path = fx.path("agent-a1");
        commit(&path, "unmerged.txt", "work on the branch\n", 0.0);
        put(&path, "wip.txt", "not committed\n");
    });
    asked(&h);
    h.svc
        .approve(&input_by(&h, &fx.path("agent-a1"), "lead"))
        .unwrap();
    h.svc.requests_idle();
    assert!(!exists_itself(&fx.path("agent-a1")));
    assert!(branches(&fx.repo).contains(&"agent-a1".to_string()));
}
