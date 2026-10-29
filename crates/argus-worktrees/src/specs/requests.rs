//! Removal requests (Node `c81fa17`, `worktrees.requests.test.mjs`) on real throwaway repositories: a lead whose own
//! session refuses `coord worktree remove` asks, and the worker removes through the SAME removal. First the guards
//! (a request has no extra power: uncommitted work and unmerged detached commits are refused with the removal's own
//! sentence, discarding needs the owner's entry, a young leftover keeps its age limit, a stranger / an ended thread /
//! the page is refused, a restart goes on with what was queued), then the happy paths (named targets with a branch,
//! `finished`, all leftovers) with the inbox note, the wake, the activity line and the summary of the project.

use crate::board::{Caller, OwnerEntry};
use crate::clock::Clock;
use crate::clock::SystemClock;
use crate::error::ErrorCode;
use crate::jobs::JobState;
use crate::paths::worktree_id;
use crate::request::RemoveInput;
use crate::requests::{
    KEPT, Mode, Queued, RequestInput, RequestRec, RequestStatus, requests_key, summary_text,
};
use crate::sync::locked;
use crate::testkit::{
    Fixture, Gate, Harness, RemoveMode, backdate, backdate_index, branches, commit, exists_itself,
    git_in, owner_now, put, started,
};
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn lead() -> Caller {
    Caller::Thread("lead".into())
}

/// A detached worktree with a commit no branch keeps: removing its folder would lose the commit.
fn detached(fx: &Fixture) {
    let path = fx.path("detached-work");
    fx.git(&["worktree", "add", "-q", "--detach", &path]);
    commit(&path, "d.txt", "detached work\n", 15.0);
    backdate_index(&path, 15.0);
}

/// The fixture with its detached worktree, one scan, and the worker started.
fn world() -> (Fixture, Harness) {
    let (fx, h) = started(detached);
    h.svc.start();
    (fx, h)
}

fn id(h: &Harness, fx: &Fixture, name: &str) -> String {
    h.id_of(&fx.path(name))
}

fn ask(
    h: &Harness,
    caller: Caller,
    change: impl FnOnce(&mut RequestInput),
) -> crate::error::Result<Queued> {
    let mut input = RequestInput::new("alpha", caller);
    change(&mut input);
    h.svc.request(&input)
}

fn ask_lead<T: AsRef<str>>(h: &Harness, targets: &[T]) -> Queued {
    let named = targets.iter().map(|t| t.as_ref().to_string()).collect();
    ask(h, lead(), |i| i.targets = named).unwrap()
}

/// Wait for the worker; the requests newest first.
fn only(h: &Harness) -> Vec<RequestRec> {
    h.svc.requests_idle();
    h.svc.request_list("alpha")
}

fn inbox(h: &Harness) -> Vec<String> {
    locked(&h.store.inbox)
        .iter()
        .map(|(_, text)| text.clone())
        .collect()
}

fn direct(h: &Harness, id: &str) -> String {
    h.svc
        .remove(&RemoveInput::new("alpha", id, lead()))
        .unwrap_err()
        .message
}

fn activity(h: &Harness, kind: &str) -> Vec<(String, String)> {
    locked(&h.notes.lines)
        .iter()
        .filter(|l| l.kind == kind)
        .map(|l| (l.level.to_string(), l.text.clone()))
        .collect()
}

#[test]
fn uncommitted_work_and_unmerged_detached_commits_are_refused_with_the_removals_own_sentence_and_the_lead_is_told()
 {
    let (fx, h) = world();
    let (dirty, det) = (id(&h, &fx, "dirty-old"), id(&h, &fx, "detached-work"));
    let (said_dirty, said_det) = (direct(&h, &dirty), direct(&h, &det));
    assert!(
        said_dirty.contains("has 1 uncommitted change: removing it discards them"),
        "{said_dirty}"
    );
    assert!(
        said_det.contains("on a detached HEAD that no branch keeps"),
        "{said_det}"
    );
    let out = ask_lead(&h, &[&dirty, &det]);
    assert_eq!(
        (out.id.as_str(), out.status),
        ("R-1", RequestStatus::Queued)
    );
    assert!(
        out.text
            .starts_with("queued R-1: Argus will remove 2 worktrees: "),
        "{}",
        out.text
    );
    assert!(
        out.text
            .contains("Nothing is removed by this command; the result comes to your inbox")
    );
    let list = only(&h);
    let r = &list[0];
    assert_eq!(r.status, RequestStatus::Refused);
    assert_eq!(
        r.results.iter().map(|x| x.ok).collect::<Vec<_>>(),
        [false, false]
    );
    assert_eq!(
        r.results[0].text, said_dirty,
        "the same sentence as the direct removal"
    );
    assert_eq!(r.results[1].text, said_det);
    assert!(exists_itself(&fx.path("dirty-old")) && exists_itself(&fx.path("detached-work")));
    let notes = inbox(&h);
    assert_eq!(notes.len(), 1);
    assert!(
        notes[0].starts_with("worktree request R-1 refused: removed 0, refused 2 ("),
        "{}",
        notes[0]
    );
}

#[test]
fn discard_needs_the_owners_page_entry_exactly_as_remove_does_and_with_it_the_work_is_backed_up_and_removed()
 {
    let (fx, h) = world();
    let target = id(&h, &fx, "dirty-old");
    ask(&h, lead(), |i| {
        (i.targets, i.discard) = (vec![target.clone()], true);
    })
    .unwrap();
    let no = only(&h);
    assert!(
        no[0].results[0]
            .text
            .contains("needs the owner: no owner entry was given"),
        "{}",
        no[0].results[0].text
    );
    assert!(exists_itself(&fx.path("dirty-old")));
    h.board().entries.insert(
        "O-1".into(),
        owner_now("O-1", "dirty-old can go, the scratch file is junk"),
    );
    ask(&h, lead(), |i| {
        (i.targets, i.discard, i.owner_entry) = (vec![target.clone()], true, Some("O-1".into()));
    })
    .unwrap();
    let yes = &only(&h)[0];
    assert_eq!(yes.status, RequestStatus::Done, "{:?}", yes.results);
    let said = &yes.results[0].text;
    assert!(
        said.starts_with("removed worktree ")
            && said.contains("dirty-old, branch dirty-old kept, backup "),
        "{said}"
    );
    assert!(said.contains(".patch"), "{said}");
    assert!(!exists_itself(&fx.path("dirty-old")));
    // An entry that is not the owner's from the page is no say, through a request either.
    h.board().entries.insert(
        "O-2".into(),
        OwnerEntry {
            author: "lead".into(),
            via: Some("cli".into()),
            ..owner_now("O-2", "detached-work can go")
        },
    );
    let det = id(&h, &fx, "detached-work");
    ask(&h, lead(), |i| {
        (i.targets, i.discard, i.owner_entry) = (vec![det], true, Some("O-2".into()));
    })
    .unwrap();
    let fake = &only(&h)[0];
    assert!(
        fake.results[0]
            .text
            .contains("O-2 was not posted by the owner from the page"),
        "{}",
        fake.results[0].text
    );
    assert!(exists_itself(&fx.path("detached-work")));
}

#[test]
fn a_young_leftover_keeps_the_age_limit_through_a_request_all_leftovers_skips_young_ones_and_says_so_and_6_hours_needs_a_reason()
 {
    let (fx, h) = world();
    let young = leftover_of(&fx, "junk-young", 0.0);
    let old = leftover_of(&fx, "junk-old", 10.0);
    h.scan();
    ask_lead(&h, &[&worktree_id(&young, h.svc.opts.case_insensitive)]);
    let one = &only(&h)[0];
    assert_eq!(one.status, RequestStatus::Refused);
    let said = &one.results[0].text;
    assert!(
        said.contains(
            ", less than 1 day ago: a thread only deletes a leftover nothing touched for 1 day"
        ),
        "{said}"
    );
    assert!(exists_itself(&young));
    let low = ask(&h, lead(), |i| {
        (i.all_leftovers, i.older_than_days) = (true, Some(0.25));
    })
    .unwrap_err();
    assert_eq!(low.code, ErrorCode::BadRequest);
    assert!(
        low.message
            .contains("below the server's limit of 1 day: say why"),
        "{}",
        low.message
    );
    ask(&h, lead(), |i| i.all_leftovers = true).unwrap();
    let all = only(&h).remove(0);
    assert_eq!(all.status, RequestStatus::Done, "{all:?}");
    assert_eq!(
        all.results.len(),
        1,
        "the old one was deleted through remove"
    );
    let deleted = &all.results[0].text;
    assert!(
        deleted.starts_with("deleted leftover folder ") && deleted.ends_with("junk-old"),
        "{deleted}"
    );
    assert!(!exists_itself(&old));
    assert!(exists_itself(&young), "the young one stays");
    assert!(all.skipped.as_ref().is_some_and(|s| s.count >= 1));
    let summary = summary_text(&all);
    assert!(
        summary.starts_with("worktree request R-2 done: removed 1; skipped "),
        "{summary}"
    );
    assert!(summary.contains("younger than 1 day"), "{summary}");
}

fn leftover_of(fx: &Fixture, name: &str, days: f64) -> String {
    let dir = fx.path(name);
    put(&dir, "a.txt", "left over\n");
    backdate(&dir, days);
    dir
}

#[test]
fn a_stranger_a_thread_that_is_not_the_lead_or_the_rules_an_ended_thread_and_the_page_are_refused_and_nothing_is_stored()
 {
    let (fx, h) = world();
    let target = id(&h, &fx, "old-merged");
    let by = |thread: &str| Caller::Thread(thread.to_string());
    for (thread, said) in [
        ("ghost", "\"ghost\" is not a live thread here"),
        ("ended", "\"ended\" is not a live thread here"),
        ("other", "other may not remove it"),
    ] {
        let err = ask(&h, by(thread), |i| i.targets = vec![target.clone()]).unwrap_err();
        assert!(
            err.code == ErrorCode::Forbidden && err.message.contains(said),
            "{thread}: {}",
            err.message
        );
    }
    let page = ask(&h, Caller::Page, |i| i.targets = vec![target.clone()]).unwrap_err();
    assert!(
        page.code == ErrorCode::Forbidden && page.message.contains("comes from a thread"),
        "{}",
        page.message
    );
    assert_eq!(h.svc.request_list("alpha").len(), 0);
    assert!(exists_itself(&fx.path("old-merged")));
    let lines = activity(&h, "worktree-request");
    assert!(
        lines.iter().any(|(level, text)| level == "warn"
            && text.contains("refused a worktree removal request from ghost")),
        "{lines:?}"
    );
    let rule_thread = ask(&h, by("worker"), |i| i.targets = vec![target.clone()]).unwrap();
    assert_eq!(
        rule_thread.status,
        RequestStatus::Queued,
        "the rule's own thread may ask"
    );
    only(&h);
}

/// The ask is refused as a bad request that says `said`.
fn refused_shape(h: &Harness, change: &dyn Fn(&mut RequestInput), said: &str) {
    let err = ask(h, lead(), |i| change(i)).unwrap_err();
    assert!(
        err.code == ErrorCode::BadRequest && err.message.contains(said),
        "{said}: {}",
        err.message
    );
}

#[test]
fn the_shape_of_a_request_is_checked_at_once_one_kind_of_ask_no_discard_for_finished_older_than_only_with_all_leftovers()
 {
    let (_fx, h) = world();
    let bad = |change: &dyn Fn(&mut RequestInput), said: &str| refused_shape(&h, change, said);
    let one = |i: &mut RequestInput| i.targets = vec!["aaaaaaaaaa".into()];
    bad(&|_| {}, "exactly one of");
    bad(
        &|i| {
            one(i);
            i.finished = true;
        },
        "exactly one of",
    );
    bad(
        &|i| (i.finished, i.discard) = (true, true),
        "never discard work",
    );
    bad(
        &|i| {
            one(i);
            i.owner_entry = Some("O-1".into());
        },
        "goes with --discard",
    );
    bad(&|i| (i.all_leftovers, i.branch) = (true, true), "no branch");
    bad(
        &|i| (i.finished, i.older_than_days) = (true, Some(3.0)),
        "goes with --all-leftovers",
    );
    bad(
        &|i| i.targets = (0..201).map(|n| format!("t{n}")).collect(),
        "200 worktrees at most (201 given)",
    );
    assert_eq!(h.svc.request_list("alpha").len(), 0);
}

fn crashed(
    base: &RequestRec,
    id: &str,
    status: RequestStatus,
    targets: &[(&str, &str)],
    results: &[Value],
) -> Value {
    let at = crate::clock::iso_from_ms(SystemClock.now_ms());
    let mut value = serde_json::to_value(base).unwrap();
    let list: Vec<Value> = targets
        .iter()
        .map(|(i, l)| json!({ "id": i, "label": l }))
        .collect();
    let finished = (status == RequestStatus::Done).then(|| at.clone());
    for (key, v) in [
        ("id", json!(id)),
        ("at", json!(at)),
        ("targets", json!(list)),
        ("results", json!(results)),
        ("status", json!(status)),
        ("startedAt", json!(at)),
        ("finishedAt", json!(finished)),
        ("notified", json!(false)),
        ("resolved", json!(true)),
    ] {
        value[key] = v;
    }
    value
}

fn crash_state(h: &Harness, fx: &Fixture, b: &str) {
    // Hand-made state of a crash: R-2 was running with its first target handled, R-3 finished and was never told.
    let key = requests_key("alpha");
    let mut state: Value = serde_json::from_str(&locked(&h.store.kv)[&key]).unwrap();
    let base = serde_json::from_value::<RequestRec>(state["list"][0].clone()).unwrap();
    let handled = json!({ "id": "ffffffffff", "label": "ffffffffff", "ok": false, "text": "handled before the crash" });
    let told = json!({ "id": "eeeeeeeeee", "label": "eeeeeeeeee", "ok": true, "text": "removed before the crash" });
    let second = crashed(
        &base,
        "R-2",
        RequestStatus::Running,
        &[("ffffffffff", "ffffffffff"), (b, &fx.path("old-unmerged"))],
        &[handled],
    );
    let third = crashed(
        &base,
        "R-3",
        RequestStatus::Done,
        &[("eeeeeeeeee", "eeeeeeeeee")],
        &[told],
    );
    state["list"]
        .as_array_mut()
        .unwrap()
        .extend([second, third]);
    state["seq"] = json!(3);
    locked(&h.store.kv).insert(key, state.to_string());
}

fn restart_of(h: &Harness, fx: &Fixture) -> Harness {
    h.svc.stop();
    let again = Harness::on_store(h.store.clone(), &fx.root, RemoveMode::Real, |_| {});
    again.svc.start();
    again
}

#[test]
fn a_restart_resumes_a_queued_request_and_a_running_one_from_the_target_it_had_not_finished_and_tells_a_finished_one_never_told()
 {
    let (fx, h) = started(|fx| {
        detached(fx);
    });
    let quiet = Harness::on_store(h.store.clone(), &fx.root, RemoveMode::Real, |o| {
        o.run_requests = false;
    });
    quiet.svc.start();
    quiet.scan();
    let (a, b) = (id(&h, &fx, "old-merged"), id(&h, &fx, "old-unmerged"));
    let queued = ask_lead(&quiet, &[&a]);
    assert_eq!(queued.status, RequestStatus::Queued);
    assert!(
        exists_itself(&fx.path("old-merged")),
        "autorun is off: nothing ran"
    );
    crash_state(&h, &fx, &b);
    quiet.svc.stop();
    let again = restart_of(&quiet, &fx);
    again.svc.requests_idle();
    let list: Vec<RequestRec> = again.svc.request_list("alpha").into_iter().rev().collect();
    let seen: Vec<(&str, RequestStatus)> = list.iter().map(|r| (r.id.as_str(), r.status)).collect();
    assert_eq!(
        seen,
        [
            ("R-1", RequestStatus::Done),
            ("R-2", RequestStatus::Partial),
            ("R-3", RequestStatus::Done)
        ]
    );
    assert!(
        !exists_itself(&fx.path("old-merged")),
        "the queued one ran after the restart"
    );
    assert!(
        !exists_itself(&fx.path("old-unmerged")),
        "the running one went on with its second target"
    );
    assert_eq!(
        list[1].results[0].text, "handled before the crash",
        "the first target was not done again"
    );
    assert_eq!(list[1].results.len(), 2);
    let texts = inbox(&again);
    assert_eq!(texts.len(), 3, "{texts:?}");
    assert!(
        texts
            .iter()
            .any(|t| t.starts_with("worktree request R-3 done: removed 1")),
        "{texts:?}"
    );
    assert!(
        locked(&again.notes.wakes)
            .iter()
            .all(|w| w.reason == "thread-message" && w.thread == "lead")
    );
    // Nothing is told twice.
    let third_start = restart_of(&again, &fx);
    third_start.svc.requests_idle();
    assert_eq!(inbox(&third_start).len(), 3);
}

#[test]
fn one_request_at_a_time_oldest_first_and_the_list_keeps_the_last_100() {
    let (fx, h) = world();
    let first = ask_lead(&h, &[&id(&h, &fx, "old-merged")]);
    let second = ask_lead(&h, &["ffffffffff"]);
    assert_eq!((first.id.as_str(), second.id.as_str()), ("R-1", "R-2"));
    let list: Vec<RequestRec> = only(&h).into_iter().rev().collect();
    let seen: Vec<RequestStatus> = list.iter().map(|r| r.status).collect();
    assert_eq!(seen, [RequestStatus::Done, RequestStatus::Refused]);
    assert!(
        list[0].finished_at <= list[1].started_at,
        "R-2 started after R-1 finished"
    );
    assert!(
        list[1].results[0]
            .text
            .contains("ffffffffff is not in the live listing"),
        "{}",
        list[1].results[0].text
    );
    // The kept list is cut at 100.
    let key = requests_key("alpha");
    let mut state: Value = serde_json::from_str(&locked(&h.store.kv)[&key]).unwrap();
    let base = state["list"][0].clone();
    let many: Vec<Value> = (0..KEPT + 5)
        .map(|n| {
            let mut rec = base.clone();
            rec["id"] = json!(format!("R-{}", n + 10));
            rec["notified"] = json!(true);
            rec
        })
        .collect();
    state["list"] = json!(many);
    state["seq"] = json!(200);
    locked(&h.store.kv).insert(key.clone(), state.to_string());
    let fresh = Harness::on_store(h.store.clone(), &fx.root, RemoveMode::Real, |o| {
        o.run_requests = false;
    });
    fresh.svc.start();
    ask_lead(&fresh, &["ffffffffff"]);
    let kept: Value = serde_json::from_str(&locked(&h.store.kv)[&key]).unwrap();
    assert_eq!(kept["list"].as_array().map(Vec::len), Some(KEPT));
}

/// The inbox note, the wake, the activity lines, the removal lines and the summary of the two-target request.
fn assert_told(h: &Harness) {
    assert_eq!(inbox(h), ["worktree request R-1 done: removed 2"]);
    let wakes = locked(&h.notes.wakes).clone();
    let wake = wakes.iter().find(|w| w.reason == "thread-message").unwrap();
    assert_eq!(
        (wake.scope.as_str(), wake.thread.as_str()),
        ("alpha", "lead")
    );
    assert_eq!(wake.text, "Argus: worktree request R-1 done: removed 2");
    let lines = activity(h, "worktree-request");
    assert!(
        lines.iter().any(|(_, t)| t
            .starts_with("alpha: worktree request R-1 queued by lead: 2 worktrees ")
            && t.ends_with(": finished agents")),
        "{lines:?}"
    );
    assert!(
        lines.contains(&(
            "info".to_string(),
            "alpha: worktree request R-1 done: removed 2 (asked by lead)".to_string()
        )),
        "{lines:?}"
    );
    let removals = h.notes.removals("removed worktree");
    assert_eq!(removals.len(), 2, "each target is an ordinary removal line");
    assert!(
        removals[0].text.ends_with("for lead: finished agents"),
        "{}",
        removals[0].text
    );
    let brief: Vec<(String, RequestStatus, usize, usize)> = h
        .meta()
        .requests
        .iter()
        .map(|x| (x.id.clone(), x.status, x.removed, x.refused))
        .collect();
    assert_eq!(brief, [("R-1".to_string(), RequestStatus::Done, 2, 0)]);
}

#[test]
fn named_worktrees_with_branch_a_merged_clean_one_goes_with_its_branch_and_the_note_the_wake_the_activity_line_and_the_summary_follow()
 {
    let (fx, h) = world();
    let (merged, unmerged) = (id(&h, &fx, "old-merged"), fx.path("old-unmerged"));
    ask(&h, lead(), |i| {
        (i.targets, i.branch, i.reason) =
            (vec![merged, unmerged], true, Some("finished agents".into()));
    })
    .unwrap();
    let r = only(&h).remove(0);
    assert_eq!(r.status, RequestStatus::Done, "{:?}", r.results);
    assert_eq!(
        r.results.iter().map(|x| x.ok).collect::<Vec<_>>(),
        [true, true]
    );
    assert!(
        r.results[0].text.starts_with("removed worktree ")
            && r.results[0]
                .text
                .contains("old-merged, branch old-merged deleted: merged into local main"),
        "{}",
        r.results[0].text
    );
    assert!(
        r.results[1]
            .text
            .contains("old-unmerged, branch old-unmerged kept (not merged)"),
        "{}",
        r.results[1].text
    );
    assert!(!exists_itself(&fx.path("old-merged")) && !exists_itself(&fx.path("old-unmerged")));
    assert!(
        branches(&fx.repo).contains(&"old-unmerged".to_string()),
        "the unmerged branch is kept"
    );
    assert_told(&h);
}

#[test]
fn finished_merged_and_stale_clean_worktrees_go_the_guards_still_refuse_the_unsafe_one_active_and_dirty_ones_are_not_even_tried()
 {
    let (fx, h) = world();
    ask(&h, lead(), |i| i.finished = true).unwrap();
    let r = only(&h).remove(0);
    assert_eq!(r.mode, Mode::Finished);
    let mut names: Vec<String> = r
        .targets
        .iter()
        .map(|t| t.label.rsplit('/').next().unwrap().to_string())
        .collect();
    names.sort();
    assert_eq!(names, ["detached-work", "old-merged", "old-unmerged"]);
    assert_eq!(r.status, RequestStatus::Partial);
    assert_eq!((r.removed(), r.refused()), (2, 1));
    let refused = r.results.iter().find(|x| !x.ok).unwrap();
    assert!(
        refused.text.contains("detached HEAD that no branch keeps"),
        "{}",
        refused.text
    );
    for kept in ["fresh", "dirty-old", "detached-work"] {
        assert!(exists_itself(&fx.path(kept)), "{kept}");
    }
    assert!(!exists_itself(&fx.path("old-merged")));
    let told = &inbox(&h)[0];
    assert!(
        told.starts_with("worktree request R-1 partial: removed 2, refused 1 (")
            && told.contains("detached-work"),
        "{told}"
    );
    // Asked again: only the unsafe one is left, and the guards refuse it again.
    ask(&h, lead(), |i| i.finished = true).unwrap();
    let again = only(&h).remove(0);
    assert_eq!(
        (again.id.as_str(), again.status, again.results.len()),
        ("R-2", RequestStatus::Refused, 1)
    );
}

#[test]
fn finished_with_nothing_finished_says_so() {
    let (fx, h) = started(|fx| {
        detached(fx);
        for name in ["old-merged", "old-unmerged", "dirty-old", "detached-work"] {
            git_in(
                &fx.repo,
                &["worktree", "remove", "--force", &fx.path(name)],
                &[],
            );
        }
    });
    h.svc.start();
    let _ = &fx;
    ask(&h, lead(), |i| i.finished = true).unwrap();
    let r = only(&h).remove(0);
    assert_eq!(
        (r.status, r.results.len(), r.note.as_deref()),
        (
            RequestStatus::Done,
            0,
            Some("no finished worktree (merged, or stale and clean) in the listing")
        )
    );
    assert!(inbox(&h)[0].starts_with("worktree request R-1 done: no finished worktree"));
}

#[test]
fn a_request_made_before_the_service_starts_waits_for_the_start() {
    let (_fx, h) = started(|_| {});
    let queued = ask_lead(&h, &["ffffffffff"]);
    assert_eq!(queued.status, RequestStatus::Queued);
    h.svc.requests_idle();
    assert_eq!(
        h.svc.request_list("alpha")[0].status,
        RequestStatus::Queued,
        "the worker is halted"
    );
    h.svc.start();
    let r = only(&h).remove(0);
    assert_eq!(r.status, RequestStatus::Refused);
    h.svc.stop();
}

/// Four extra worktrees at the tip of main or beside it: `idle-old` (clean, 0 ahead, newest change 2 days ago),
/// `idle-young` (the same, touched now), `idle-ahead` (a commit main lacks) and `idle-dirty` (an untracked file).
fn idle_prep(fx: &Fixture) {
    detached(fx);
    fx.add("idle-young");
    backdate_index(&fx.add("idle-old"), 2.0);
    let ahead = fx.add("idle-ahead");
    commit(&ahead, "ahead.txt", "work main lacks\n", 2.0);
    backdate_index(&ahead, 2.0);
    let dirty = fx.add("idle-dirty");
    put(&dirty, "scratch.txt", "not committed\n");
    backdate_index(&dirty, 2.0);
}

fn names(r: &RequestRec) -> Vec<String> {
    let mut names: Vec<String> = r
        .targets
        .iter()
        .map(|t| t.label.rsplit('/').next().unwrap_or_default().to_string())
        .collect();
    names.sort();
    names
}

fn ask_idle(h: &Harness, minutes: f64) -> Queued {
    ask(h, lead(), |i| {
        (i.finished, i.idle_minutes) = (true, Some(minutes));
    })
    .unwrap()
}

#[test]
fn finished_idle_also_takes_a_clean_worktree_with_nothing_ahead_older_than_d_and_never_a_younger_dirty_or_unmerged_one()
 {
    let (fx, h) = started(idle_prep);
    h.svc.start();
    ask_idle(&h, 24.0 * 60.0);
    let r = only(&h).remove(0);
    assert_eq!(r.idle_minutes, Some(1440.0));
    assert_eq!(
        names(&r),
        ["detached-work", "idle-old", "old-merged", "old-unmerged"]
    );
    assert!(
        r.results
            .iter()
            .any(|x| x.ok && x.text.contains("idle-old")),
        "idle-old was removed through remove"
    );
    assert!(!exists_itself(&fx.path("idle-old")));
    for kept in [
        "idle-young",
        "idle-ahead",
        "idle-dirty",
        "fresh",
        "dirty-old",
    ] {
        assert!(exists_itself(&fx.path(kept)), "{kept} stays");
    }
    let lines = activity(&h, "worktree-request");
    assert!(
        lines.iter().any(
            |(_, t)| t.contains("queued by lead: every finished worktree of alpha ")
                && t.contains(" and every idle clean worktree older than 1d ")
        ),
        "{lines:?}"
    );
}

#[test]
fn finished_idle_the_age_is_the_cut_90_minutes_takes_the_2_day_one_6_hours_leaves_the_one_touched_now()
 {
    let (fx, h) = started(idle_prep);
    h.svc.start();
    ask_idle(&h, 3.0 * 24.0 * 60.0);
    let none = only(&h).remove(0);
    assert_eq!(
        names(&none),
        ["detached-work", "old-merged", "old-unmerged"],
        "idle-old (2 days) is younger than 3 days"
    );
    let out = ask_idle(&h, 90.0);
    assert!(
        out.text
            .contains("every idle clean worktree older than 90m"),
        "{}",
        out.text
    );
    let mins = only(&h).remove(0);
    assert_eq!(mins.idle_minutes, Some(90.0));
    assert_eq!(names(&mins), ["detached-work", "idle-old"]);
    assert!(
        exists_itself(&fx.path("idle-young")),
        "idle-young was touched just now"
    );
    let six = ask_idle(&h, 360.0);
    assert!(six.text.contains("older than 6h"), "{}", six.text);
    let hours = only(&h).remove(0);
    assert_eq!(
        names(&hours),
        ["detached-work"],
        "only the refused one again"
    );
    assert!(exists_itself(&fx.path("idle-young")));
}

#[test]
fn plain_finished_is_unchanged_by_the_idle_worktrees_and_the_shape_of_idle_is_checked_at_once() {
    let (fx, h) = started(idle_prep);
    h.svc.start();
    ask(&h, lead(), |i| i.finished = true).unwrap();
    let r = only(&h).remove(0);
    assert_eq!(r.idle_minutes, None);
    assert_eq!(names(&r), ["detached-work", "old-merged", "old-unmerged"]);
    for kept in ["idle-young", "idle-old", "idle-ahead", "idle-dirty"] {
        assert!(exists_itself(&fx.path(kept)), "{kept} stays");
    }
    let bad = |change: &dyn Fn(&mut RequestInput), said: &str| refused_shape(&h, change, said);
    bad(
        &|i| (i.finished, i.idle_minutes) = (true, Some(30.0)),
        "--idle takes a time from 1h to 30d (6h, 2d, 90m)",
    );
    bad(
        &|i| (i.finished, i.idle_minutes) = (true, Some(31.0 * 1440.0)),
        "--idle takes a time from 1h to 30d",
    );
    bad(
        &|i| (i.finished, i.idle_minutes) = (true, Some(f64::NAN)),
        "--idle takes a time",
    );
    bad(
        &|i| (i.all_leftovers, i.idle_minutes) = (true, Some(360.0)),
        "--idle goes with --finished",
    );
    let named = id(&h, &fx, "old-merged");
    bad(
        &|i| (i.targets, i.idle_minutes) = (vec![named.clone()], Some(360.0)),
        "--idle goes with --finished",
    );
    assert_eq!(
        h.svc.request_list("alpha").len(),
        1,
        "refused asks store nothing"
    );
}

/// Waits up to a minute for `done`.
fn within_a_minute(what: &str, done: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(60);
    while !done() {
        assert!(Instant::now() < deadline, "{what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The live bug of 2026-10-29: Brock's 88 leftovers (10 minutes each) held Tessera's request for hours. Each project has
/// its own worker now: while alpha's deletion is held, beta's request by path (beta was never scanned) is done.
#[test]
fn a_long_request_of_one_project_does_not_hold_up_a_request_by_path_of_another() {
    let (fx, h) = world();
    let slow = leftover_of(&fx, "left-slow", 3.0);
    h.scan();
    let gate = Arc::new(Gate::default());
    *locked(&h.fs.gate) = Some(gate.clone());
    *locked(&h.fs.gate_on) = Some("left-slow".to_string());
    let held = ask_lead(&h, &[&slow]);
    within_a_minute("alpha's deletion is running", || {
        h.svc
            .jobs("alpha")
            .iter()
            .any(|j| j.state == JobState::Running)
    });
    locked(&h.store.projects).push("beta".to_string());
    let mut input = RequestInput::new("beta", lead());
    input.targets = vec![fx.path("old-merged")];
    h.svc.request(&input).unwrap();
    within_a_minute("beta's request is done while alpha's waits", || {
        h.svc
            .request_list("beta")
            .first()
            .is_some_and(|r| r.status == RequestStatus::Done)
    });
    assert!(!exists_itself(&fx.path("old-merged")));
    assert_eq!(
        h.svc.request_list("alpha")[0].status,
        RequestStatus::Running
    );
    gate.open();
    let alpha = only(&h).remove(0);
    assert_eq!(
        (alpha.id, alpha.status),
        (held.id, RequestStatus::Done),
        "{:?}",
        alpha.results
    );
    assert!(locked(&h.notes.problems).is_empty());
}
