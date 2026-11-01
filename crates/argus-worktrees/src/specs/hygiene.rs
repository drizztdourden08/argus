//! Worktree hygiene on real throwaway repositories: a sub-agent's worktree is released when its agent ended, nothing
//! links it and removing it is safe; its thread hears it once (one `worktree-review` wake per thread, a burst makes
//! one), and after the grace Argus removes it through the same removal and leaves one inbox note. Each guard that
//! blocks a release has its test, then the keep, the thread's own release, the look again right before removing,
//! and the cases that are never automatic.

use crate::board::{AgentFact, Caller, Keep, Link};
use crate::clock::Clock;
use crate::hygiene::HygieneState;
use crate::paths::join_path;
use crate::ports::ProcInfo;
use crate::request::RemoveInput;
use crate::rule::Rule;
use crate::sync::locked;
use crate::testkit::{
    Fixture, Harness, RemoveMode, branches, commit, exists_itself, make_link, put, rule_for,
};
use argus_core::fixture::links::{assert_sibling_intact, fill_sibling};
use std::path::Path;

const GRACE: i64 = 3_600_000;
const QUIET: i64 = 30_000;

fn agent(n: &str, thread: &str, status: &str, ended_ms: i64) -> AgentFact {
    AgentFact {
        id: format!("A-{n}"),
        thread: Some(thread.to_string()),
        status: status.to_string(),
        worktree: None,
        claude_id: Some(format!("a{n}")),
        ended_ms: Some(ended_ms),
    }
}

/// A service over a fixture with one clean worktree `agent-a<n>` per agent (at the tip of main: merged), the agents on
/// the board, one scan, and the request worker started.
fn world_with(agents: &[(&str, &str, &str)], prep: impl FnOnce(&Fixture)) -> (Fixture, Harness) {
    let fx = Fixture::new();
    for (n, _, _) in agents {
        fx.add(&format!("agent-a{n}"));
    }
    prep(&fx);
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |o| {
        o.release_debounce_ms = QUIET;
        o.release_grace_ms = GRACE;
    });
    let ended = h.clock_now() - 1000;
    h.board().agents = agents
        .iter()
        .map(|(n, thread, status)| agent(n, thread, status, ended))
        .collect();
    h.scan();
    h.svc.start();
    (fx, h)
}

fn world(prep: impl FnOnce(&Fixture)) -> (Fixture, Harness) {
    world_with(&[("1", "lead", "done")], prep)
}

fn state(h: &Harness, name: &str) -> Option<(HygieneState, Option<String>)> {
    h.docs()
        .get(name)
        .and_then(|d| d.hygiene.clone())
        .map(|g| (g.state, g.why))
}

/// The reason the worktree is in review; fails the test when it is not.
fn review_why(h: &Harness, name: &str) -> String {
    match state(h, name) {
        Some((HygieneState::Review, Some(why))) => why,
        other => panic!("{name} should be in review, it is {other:?}"),
    }
}

fn step(h: &Harness, ms: i64) -> usize {
    h.clock.advance(ms);
    h.svc.hygiene_step("alpha").told
}

fn reviews(h: &Harness) -> Vec<(String, String)> {
    locked(&h.notes.wakes)
        .iter()
        .filter(|w| w.reason == "worktree-review")
        .map(|w| (w.thread.clone(), w.text.clone()))
        .collect()
}

fn inbox(h: &Harness) -> Vec<(String, String)> {
    locked(&h.store.inbox).clone()
}

/// Told, then the grace over: the request runs. Returns whether the folder is gone.
fn through_grace(h: &Harness, path: &str) -> bool {
    step(h, QUIET);
    step(h, GRACE);
    h.svc.requests_idle();
    !exists_itself(path)
}

fn link(what: &str) -> Link {
    Link {
        what: what.to_string(),
        ..Link::default()
    }
}

/// The worktree of agent 1 with something added to the board or the machine, judged after a fresh scan.
fn blocked_by(change: impl FnOnce(&Fixture, &Harness)) -> String {
    let (fx, h) = world(|_| {});
    assert_eq!(
        state(&h, "agent-a1").map(|s| s.0),
        Some(HygieneState::Released)
    );
    change(&fx, &h);
    h.scan();
    let why = review_why(&h, "agent-a1");
    assert!(
        !through_grace(&h, &fx.path("agent-a1")),
        "{why}: it must stay"
    );
    why
}

#[test]
fn a_finished_agents_clean_merged_worktree_is_released_at_once() {
    let (fx, h) = world(|_| {});
    assert_eq!(state(&h, "agent-a1"), Some((HygieneState::Released, None)));
    let row = &h.docs()["agent-a1"];
    let hygiene = row.hygiene.as_ref().unwrap();
    assert_eq!(
        (
            hygiene.agent.as_deref(),
            hygiene.thread.as_deref(),
            hygiene.goes_at.as_deref()
        ),
        (Some("A-1"), Some("lead"), None)
    );
    assert_eq!(
        state(&h, "fresh"),
        None,
        "no agent owns it: the hygiene says nothing"
    );
    assert_eq!(state(&h, "dirty-old"), None);
    assert!(
        reviews(&h).is_empty(),
        "the wake waits for the burst to end"
    );
    assert_eq!(step(&h, QUIET), 1);
    let (thread, text) = reviews(&h).pop().unwrap();
    assert_eq!(thread, "lead");
    assert!(
        text.contains(&fx.path("agent-a1")) && text.contains("in 1 hour"),
        "{text}"
    );
    assert!(
        h.docs()["agent-a1"]
            .hygiene
            .as_ref()
            .unwrap()
            .goes_at
            .is_some()
    );
    assert_eq!(step(&h, QUIET), 0, "told once");
}

#[test]
fn an_agent_still_at_work_holds_its_worktree() {
    let (_fx, h) = world_with(&[("1", "lead", "working")], |_| {});
    assert_eq!(state(&h, "agent-a1"), None);
    for status in ["waiting", "stop-requested"] {
        h.board().agents[0].status = status.to_string();
        h.svc.hygiene_step("alpha");
        assert_eq!(state(&h, "agent-a1"), None, "{status}");
    }
}

#[test]
fn a_failed_agent_is_review_not_released() {
    let (_fx, h) = world_with(&[("1", "lead", "failed")], |_| {});
    assert!(review_why(&h, "agent-a1").contains("its agent A-1 is failed"));
}

#[test]
fn an_agent_that_ended_after_the_scan_waits_for_a_fresh_look() {
    let (_fx, h) = world(|_| {});
    let _ = h.svc.take_rescans();
    h.board().agents[0].ended_ms = Some(h.clock_now() + 5000);
    h.svc.hygiene_step("alpha");
    assert!(review_why(&h, "agent-a1").contains("looks at it again first"));
    assert_eq!(h.svc.take_rescans(), vec!["alpha".to_string()]);
}

#[test]
fn the_record_that_names_the_worktree_owns_it_too() {
    let (fx, h) = world(|fx| {
        fx.add("named-work");
    });
    let ended = h.clock_now() - 1000;
    h.board().agents.push(AgentFact {
        worktree: Some(fx.path("named-work")),
        claude_id: None,
        ..agent("2", "lead", "done", ended)
    });
    h.svc.hygiene_step("alpha");
    assert_eq!(
        state(&h, "named-work").map(|s| s.0),
        Some(HygieneState::Released)
    );
}

#[test]
fn a_keep_holds_it() {
    let (fx, h) = world(|_| {});
    step(&h, QUIET);
    let id = h.id_of(&fx.path("agent-a1"));
    let until_ms = h.clock_now() + 86_400_000;
    h.board().keeps.insert(
        id,
        Keep {
            until_ms,
            note: Some("still testing".into()),
            by: Some("lead".into()),
        },
    );
    h.svc.hygiene_step("alpha");
    assert_eq!(
        state(&h, "agent-a1"),
        Some((HygieneState::Kept, Some("still testing".into())))
    );
    step(&h, GRACE);
    h.svc.requests_idle();
    assert!(exists_itself(&fx.path("agent-a1")) && h.svc.request_list("alpha").is_empty());
}

#[test]
fn an_open_item_that_names_it_blocks_it() {
    let why = blocked_by(|_, h| {
        h.board().links.push(Link {
            text: Some("land the agent-a1 work first".into()),
            ..link("item I-4")
        });
    });
    assert_eq!(why, "item I-4 uses it");
}

#[test]
fn an_open_pull_request_of_its_branch_blocks_it() {
    let why = blocked_by(|_, h| {
        h.board().links.push(Link {
            branch: Some("agent-a1".into()),
            ..link("PR-2")
        });
    });
    assert_eq!(why, "PR-2 uses it");
}

#[test]
fn a_launcher_in_it_blocks_it() {
    let why = blocked_by(|fx, h| {
        h.board().links.push(Link {
            path: Some(fx.path("agent-a1")),
            ..link("launcher web")
        });
    });
    assert_eq!(why, "launcher web uses it");
}

#[test]
fn a_bundle_inside_it_blocks_it() {
    let why = blocked_by(|fx, h| {
        h.board().links.push(Link {
            path: Some(join_path(&fx.path("agent-a1"), "apps/web")),
            ..link("bundle b1")
        });
    });
    assert_eq!(why, "bundle b1 uses it");
}

#[test]
fn a_run_or_check_run_in_it_blocks_it_and_unreadable_runs_block_everything() {
    let why = blocked_by(|fx, h| {
        *locked(&h.store.runs) = Ok(vec![Link {
            path: Some(fx.path("agent-a1")),
            ..link("run R-7")
        }]);
    });
    assert_eq!(why, "run R-7 uses it");
    let why = blocked_by(|_, h| *locked(&h.store.runs) = Err("database is locked".into()));
    assert!(
        why.contains("the runs could not be read (database is locked)"),
        "{why}"
    );
}

#[test]
fn a_process_in_it_blocks_it() {
    let why = blocked_by(|fx, h| {
        h.clock.advance(10_001);
        locked(&h.procs.0).push(ProcInfo {
            pid: 4242,
            name: "node.exe".into(),
            cwd: Some(fx.path("agent-a1")),
            ..ProcInfo::default()
        });
    });
    assert_eq!(why, "node.exe 4242 uses it");
}

#[test]
fn uncommitted_work_is_never_automatic() {
    let why = blocked_by(|fx, _| {
        put(&fx.path("agent-a1"), "notes.txt", "not committed\n");
    });
    assert_eq!(why, "it has 1 uncommitted change");
}

#[test]
fn commits_no_ref_keeps_are_never_automatic() {
    let (fx, h) = world_with(&[], |fx| {
        let path = fx.path("agent-a1");
        fx.git(&["worktree", "add", "-q", "--detach", &path]);
        commit(&path, "d.txt", "detached work\n", 0.0);
    });
    h.board().agents = vec![agent("1", "lead", "done", h.clock_now() - 1000)];
    h.scan();
    assert_eq!(
        review_why(&h, "agent-a1"),
        "its HEAD has commits no branch or ref keeps"
    );
    assert!(!through_grace(&h, &fx.path("agent-a1")));
}

#[test]
fn a_lock_is_never_automatic_but_brocks_own_lock_is() {
    let why = blocked_by(|fx, _| {
        fx.git(&[
            "worktree",
            "lock",
            "--reason",
            "owner: hands off",
            &fx.path("agent-a1"),
        ]);
    });
    assert_eq!(why, "it is locked (owner: hands off)");
    let (fx, h) = world(|fx| {
        fx.git(&[
            "worktree",
            "lock",
            "--reason",
            "cloned node_modules from X:/brock",
            &fx.path("agent-a1"),
        ]);
    });
    assert_eq!(
        state(&h, "agent-a1").map(|s| s.0),
        Some(HygieneState::Released)
    );
    assert!(
        through_grace(&h, &fx.path("agent-a1")),
        "the removal lifts Brock's lock"
    );
}

#[test]
fn a_main_checkout_is_never_automatic() {
    let fx = Fixture::new();
    let main_rule = Rule::folder("W-2", &fx.root, "worker").filtered(&["proj"], &[]);
    let h = Harness::new(&fx.root, vec![main_rule], |o| o.release_grace_ms = GRACE);
    h.board().agents = vec![AgentFact {
        worktree: Some(fx.repo.clone()),
        claude_id: None,
        ..agent("1", "lead", "done", h.clock_now() - 1000)
    }];
    h.scan();
    h.svc.start();
    assert!(review_why(&h, "proj").contains("main checkout"));
    step(&h, GRACE * 2);
    h.svc.requests_idle();
    assert_eq!(h.svc.request_list("alpha").len(), 0);
}

#[test]
fn after_the_grace_argus_removes_it_with_its_merged_branch_and_leaves_one_note() {
    let (fx, h) = world_with(&[("1", "lead", "done"), ("2", "lead", "done")], |_| {});
    let (one, two) = (fx.path("agent-a1"), fx.path("agent-a2"));
    step(&h, QUIET);
    step(&h, GRACE - 1);
    h.svc.requests_idle();
    assert!(
        h.svc.request_list("alpha").is_empty(),
        "the grace is not over"
    );
    step(&h, 1);
    h.svc.requests_idle();
    assert!(!exists_itself(&one) && !exists_itself(&two));
    let left = branches(&fx.repo);
    assert!(
        !left.contains(&"agent-a1".to_string()) && !left.contains(&"agent-a2".to_string()),
        "{left:?}"
    );
    let notes = inbox(&h);
    assert_eq!(notes.len(), 1, "{notes:?}");
    let (thread, text) = &notes[0];
    assert_eq!(thread, "lead");
    assert!(
        text.starts_with("released worktrees, their grace over (request R-1 done): removed 2 ("),
        "{text}"
    );
    assert!(text.contains(&one), "{text}");
    let removed = h.meta().hygiene.removed;
    assert_eq!(removed.len(), 2);
    assert!(removed.iter().all(|r| r.thread.as_deref() == Some("lead")));
    assert_eq!(state(&h, "agent-a1"), None);
}

#[test]
fn a_file_made_dirty_during_the_grace_stops_the_removal() {
    let (fx, h) = world(|_| {});
    let path = fx.path("agent-a1");
    step(&h, QUIET);
    put(&path, "late.txt", "written during the grace\n");
    assert!(
        !through_grace(&h, &path),
        "the look again right before removing sees it"
    );
    assert!(exists_itself(&join_path(&path, "late.txt")));
    let (_, text) = inbox(&h).pop().unwrap();
    assert!(
        text.contains("was not removed: it has 1 uncommitted change"),
        "{text}"
    );
    assert!(review_why(&h, "agent-a1").contains("uncommitted"));
}

#[test]
fn a_burst_of_agents_ending_makes_one_wake_per_thread() {
    let (fx, h) = world_with(
        &[
            ("1", "lead", "done"),
            ("2", "lead", "working"),
            ("3", "lead", "working"),
            ("4", "worker", "done"),
        ],
        |_| {},
    );
    for n in [1, 2] {
        step(&h, 10_000);
        let ended = h.clock_now() - 1;
        let mut board = h.board();
        board.agents[n].status = "done".into();
        board.agents[n].ended_ms = Some(ended);
        drop(board);
        h.scan();
    }
    assert_eq!(
        step(&h, QUIET - 1),
        1,
        "the worker's one worktree is told, the lead's burst is not over"
    );
    assert_eq!(step(&h, 1), 1);
    assert_eq!(step(&h, QUIET), 0);
    let mut wakes = reviews(&h);
    wakes.sort();
    assert_eq!(wakes.len(), 2);
    assert_eq!(wakes[0].0, "lead");
    assert!(
        wakes[0]
            .1
            .starts_with("3 worktrees of ended sub-agents are released"),
        "{}",
        wakes[0].1
    );
    assert_eq!(wakes[1].0, "worker");
    assert!(wakes[1].1.contains(&fx.path("agent-a4")));
}

#[test]
fn the_threads_own_release_removes_it_at_once_and_refuses_what_is_not_safe() {
    let (fx, h) = world_with(
        &[("1", "lead", "working"), ("2", "lead", "working")],
        |_| {},
    );
    let release = |name: &str| {
        let id = h.id_of(&fx.path(name));
        h.svc.release(&RemoveInput::new(
            "alpha",
            &id,
            Caller::Thread("lead".into()),
        ))
    };
    let out = release("agent-a1").unwrap();
    assert!(
        out.removed && !exists_itself(&fx.path("agent-a1")),
        "{}",
        out.text
    );
    assert_eq!(h.meta().hygiene.removed[0].path, fx.path("agent-a1"));
    put(&fx.path("agent-a2"), "wip.txt", "work\n");
    let err = release("agent-a2").unwrap_err();
    assert!(
        err.message
            .contains("is not released: it has 1 uncommitted change"),
        "{}",
        err.message
    );
    assert!(exists_itself(&fx.path("agent-a2")));
    let page = h.svc.release(&RemoveInput::new(
        "alpha",
        &h.id_of(&fx.path("agent-a2")),
        Caller::Page,
    ));
    assert!(
        page.unwrap_err()
            .message
            .contains("release is a thread's word")
    );
    let stranger = RemoveInput::new(
        "alpha",
        &h.id_of(&fx.path("agent-a2")),
        Caller::Thread("other".into()),
    );
    assert!(
        h.svc
            .release(&stranger)
            .unwrap_err()
            .message
            .contains("other may not remove it")
    );
}

#[test]
fn a_junction_inside_is_removed_as_a_link_and_its_target_stays() {
    let fx = Fixture::new();
    let path = fx.add("agent-a1");
    put(&fx.repo, ".git/info/exclude", "lnk\n");
    let sibling = join_path(&fx.root, "sibling");
    fill_sibling(Path::new(&sibling));
    make_link(&join_path(&path, "lnk"), &sibling);
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |o| {
        o.release_grace_ms = GRACE;
        o.release_debounce_ms = QUIET;
        o.drop_links_first = true;
    });
    h.board().agents = vec![agent("1", "lead", "done", h.clock_now() - 1000)];
    h.scan();
    h.svc.start();
    assert_eq!(
        state(&h, "agent-a1").map(|s| s.0),
        Some(HygieneState::Released)
    );
    assert!(through_grace(&h, &path));
    assert_sibling_intact(Path::new(&sibling));
}

#[test]
fn a_restart_keeps_when_it_goes() {
    let (fx, h) = world(|_| {});
    step(&h, QUIET);
    let goes = h.docs()["agent-a1"].hygiene.clone().unwrap().goes_at;
    let again = Harness::on_store(h.store.clone(), &fx.root, RemoveMode::Real, |o| {
        o.release_grace_ms = GRACE;
    });
    again.clock.set(h.clock.now_ms());
    again.scan();
    assert_eq!(
        again.docs()["agent-a1"].hygiene.clone().unwrap().goes_at,
        goes
    );
}
