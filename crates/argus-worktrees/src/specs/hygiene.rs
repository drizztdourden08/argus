//! KEEP: K3, K4.
//! Worktree hygiene on the fake git, the part with nothing left to lose: a worktree is attached while a running agent
//! owns it, a link or a run is in it or a process uses it; once nothing is attached and nothing would be lost, Argus
//! removes it after the debounce through the same removal and leaves one inbox note per lead. Each thing that attaches
//! a worktree has its test, then the look again right before removing, an agent or a run that comes back, the thread's
//! own release, the board keep that no longer holds it, and the cases the hygiene never touches.

// test-tag: standard

use crate::board::{AgentFact, Caller, Link};
use crate::clock::Clock;
use crate::hygiene::HygieneState;
use crate::paths::join_path;
use crate::request::RemoveInput;
use crate::rule::Rule;
use crate::specs::hygiene_kit::{
    DEBOUNCE, agent, agents_rule, attached_why, hygiene_on, hygienic, inbox, kind, left_why, state,
    step, through_debounce, world, world_with,
};
use crate::sync::locked;
use crate::testkit::{
    Fixture, Harness, RemoveMode, branches, exists_itself, make_link, put, rule_for,
};
use argus_core::fixture::links::{assert_sibling_intact, fill_sibling};
use std::path::Path;

fn link(what: &str) -> Link {
    Link {
        what: what.to_string(),
        ..Link::default()
    }
}

/// The worktree of agent 1 with something attached to it on the board or the machine, judged after a fresh scan: why
/// it is attached. It must stay past the debounce.
fn attached_by(change: impl FnOnce(&Fixture, &Harness)) -> String {
    let (fx, h) = world(|_| {});
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Removing));
    change(&fx, &h);
    h.scan();
    let why = attached_why(&h, "agent-a1");
    assert!(!through_debounce(&h, &fx.path("agent-a1")), "{why}: it must stay");
    why
}

#[test]
fn a_clean_worktree_nothing_is_attached_to_goes_after_the_debounce() {
    let (fx, h) = world(|_| {});
    assert_eq!(state(&h, "agent-a1"), Some((HygieneState::Removing, None)));
    let hygiene = h.docs()["agent-a1"].hygiene.clone().unwrap();
    assert_eq!(
        (hygiene.agent.as_deref(), hygiene.thread.as_deref()),
        (Some("A-1"), Some("lead"))
    );
    assert!(hygiene.goes_at.is_some());
    assert_eq!(state(&h, "fresh"), None, "the rule does not list it");
    step(&h, DEBOUNCE - 1);
    h.svc.requests_idle();
    assert!(h.svc.request_list("alpha").is_empty(), "the debounce is not over");
    assert!(through_debounce(&h, &fx.path("agent-a1")));
    assert_eq!(state(&h, "agent-a1"), None);
}

#[test]
fn the_removal_takes_the_merged_branches_and_leaves_one_note_per_lead() {
    let (fx, h) = world_with(&[("1", "lead", "done"), ("2", "lead", "done")], |_| {});
    let (one, two) = (fx.path("agent-a1"), fx.path("agent-a2"));
    assert!(through_debounce(&h, &one) && !exists_itself(&two));
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
        text.starts_with(
            "worktrees nothing was attached to, removed by Argus (request R-1 done): removed 2 ("
        ),
        "{text}"
    );
    assert!(text.contains(&one), "{text}");
    let removed = h.meta().hygiene.removed;
    assert_eq!(removed.len(), 2);
    let request = &h.svc.request_list("alpha")[0];
    assert!(request.branch && !request.discard, "{request:?}");
}

#[test]
fn a_running_agent_attaches_its_worktree() {
    let (fx, h) = world_with(&[("1", "lead", "working")], |_| {});
    assert_eq!(attached_why(&h, "agent-a1"), "its agent A-1 is working");
    for status in ["waiting", "stop-requested"] {
        h.board().agents[0].status = status.to_string();
        h.svc.hygiene_step("alpha");
        assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Attached), "{status}");
    }
    assert!(!through_debounce(&h, &fx.path("agent-a1")));
}

#[test]
fn a_running_agent_owns_the_folder_named_after_its_claude_id_though_its_record_names_no_worktree() {
    let (_fx, h) = world_with(&[("1", "lead", "working"), ("2", "lead", "done")], |_| {});
    assert_eq!(h.board().agents[0].worktree, None);
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Attached));
    assert_eq!(kind(&h, "agent-a2"), Some(HygieneState::Removing));
}

#[test]
fn a_failed_agent_leaves_its_clean_worktree_unattached() {
    let (_fx, h) = world_with(&[("1", "lead", "failed")], |_| {});
    assert_eq!(state(&h, "agent-a1"), Some((HygieneState::Removing, None)));
}

#[test]
fn a_worktree_no_agent_record_names_is_judged_whatever_its_age() {
    let fx = Fixture::fake();
    let h = hygienic(&fx, rule_for(&fx));
    h.scan();
    h.svc.start();
    for name in ["fresh", "old-unmerged", "old-merged"] {
        assert_eq!(state(&h, name), Some((HygieneState::Removing, None)), "{name}");
    }
    assert!(left_why(&h, "dirty-old").contains("uncommitted"));
    assert_eq!(state(&h, "plain"), None, "a leftover that holds a file");
    step(&h, DEBOUNCE);
    h.svc.requests_idle();
    for name in ["fresh", "old-unmerged", "old-merged"] {
        assert!(!exists_itself(&fx.path(name)), "{name} should be gone");
    }
    assert!(exists_itself(&fx.path("dirty-old")), "it waits for its lead");
    assert!(branches(&fx.repo).contains(&"old-unmerged".to_string()));
    let (thread, _) = inbox(&h)
        .into_iter()
        .find(|(_, t)| t.starts_with("worktrees nothing was attached to"))
        .unwrap();
    assert_eq!(thread, "worker", "the rule's thread");
}

#[test]
fn an_empty_leftover_folder_goes_through_the_leftover_removal() {
    let fx = Fixture::fake();
    let empty = fx.path("agent-gone");
    std::fs::create_dir_all(&empty).unwrap();
    let h = hygienic(&fx, rule_for(&fx));
    h.scan();
    h.svc.start();
    assert_eq!(state(&h, "agent-gone"), Some((HygieneState::Removing, None)));
    assert_eq!(state(&h, "plain"), None, "a folder that holds a file stays");
    assert!(through_debounce(&h, &empty));
    assert!(exists_itself(&fx.path("plain")));
}

#[test]
fn an_agent_that_ends_asks_for_a_sweep_and_the_counter_counts_the_running_ones() {
    let (_fx, h) = world_with(&[("1", "lead", "working"), ("2", "lead", "working")], |_| {});
    assert!(!h.svc.agents_ended("alpha"), "the first look only remembers");
    assert!(!h.svc.agents_ended("alpha"));
    let meta = h.meta();
    assert_eq!((meta.listed, meta.running_agents), (2, 2));
    h.board().agents[0].status = "done".into();
    assert!(h.svc.agents_ended("alpha"), "agent 1 is done");
    assert!(!h.svc.agents_ended("alpha"), "told once");
    h.board().agents[1].status = "stopped".into();
    assert!(h.svc.agents_ended("alpha"), "agent 2 is stopped");
    h.svc.hygiene_step("alpha");
    let meta = h.svc.derive("alpha").unwrap().meta;
    assert_eq!((meta.listed, meta.running_agents), (2, 0));
}

#[test]
fn an_agent_that_ended_after_the_scan_stays_attached_until_a_fresh_look() {
    let (_fx, h) = world(|_| {});
    let _ = h.svc.take_rescans();
    h.board().agents[0].ended_ms = Some(h.clock_now() + 5000);
    h.svc.hygiene_step("alpha");
    assert!(attached_why(&h, "agent-a1").contains("looks at it again first"));
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
        ..agent("2", "lead", "working", ended)
    });
    h.svc.hygiene_step("alpha");
    assert_eq!(attached_why(&h, "named-work"), "its agent A-2 is working");
}

#[test]
fn an_open_item_that_names_it_attaches_it() {
    let why = attached_by(|_, h| {
        h.board().links.push(Link {
            text: Some("land the agent-a1 work first".into()),
            ..link("item I-4")
        });
    });
    assert_eq!(why, "item I-4 works in it");
}

#[test]
fn a_launcher_or_a_bundle_in_it_attaches_it() {
    let why = attached_by(|fx, h| {
        h.board().links.push(Link {
            path: Some(fx.path("agent-a1")),
            ..link("launcher web")
        });
    });
    assert_eq!(why, "launcher web works in it");
    let why = attached_by(|fx, h| {
        h.board().links.push(Link {
            path: Some(join_path(&fx.path("agent-a1"), "apps/web")),
            ..link("bundle b1")
        });
    });
    assert_eq!(why, "bundle b1 works in it");
}

#[test]
fn a_run_or_check_run_in_it_attaches_it_and_unreadable_runs_attach_everything() {
    let why = attached_by(|fx, h| {
        *locked(&h.store.runs) = Ok(vec![Link {
            path: Some(fx.path("agent-a1")),
            ..link("run R-7")
        }]);
    });
    assert_eq!(why, "run R-7 works in it");
    let why = attached_by(|_, h| *locked(&h.store.runs) = Err("database is locked".into()));
    assert_eq!(why, "the runs could not be read (database is locked)");
}

#[test]
fn a_process_in_it_attaches_it() {
    let why = attached_by(|fx, h| {
        h.clock.advance(10_001);
        h.run_node_in(&fx.path("agent-a1"));
    });
    assert_eq!(why, "node.exe 4242 uses it");
}

#[test]
fn a_run_that_starts_in_the_debounce_stops_the_removal_and_its_end_starts_the_debounce_again() {
    let (fx, h) = world(|_| {});
    let path = fx.path("agent-a1");
    step(&h, DEBOUNCE - 1000);
    *locked(&h.store.runs) = Ok(vec![Link {
        path: Some(path.clone()),
        ..link("run R-7")
    }]);
    step(&h, 1000);
    h.svc.requests_idle();
    assert_eq!(attached_why(&h, "agent-a1"), "run R-7 works in it");
    assert!(exists_itself(&path) && h.svc.request_list("alpha").is_empty());
    *locked(&h.store.runs) = Ok(Vec::new());
    step(&h, DEBOUNCE - 1);
    h.svc.requests_idle();
    assert!(exists_itself(&path), "a new debounce began when the run ended");
    step(&h, 1);
    h.svc.requests_idle();
    assert!(!exists_itself(&path));
}

#[test]
fn a_file_made_dirty_during_the_debounce_stops_the_removal() {
    let (fx, h) = world(|_| {});
    let path = fx.path("agent-a1");
    put(&path, "late.txt", "written during the debounce\n");
    assert!(
        !through_debounce(&h, &path),
        "the look again right before removing sees it"
    );
    assert!(exists_itself(&join_path(&path, "late.txt")));
    let (_, text) = inbox(&h).pop().unwrap();
    assert!(
        text.contains("was not removed") && text.contains("uncommitted"),
        "{text}"
    );
    h.scan();
    assert!(left_why(&h, "agent-a1").contains("uncommitted"));
}

#[test]
fn a_keep_a_thread_wrote_on_the_board_does_not_hold_it() {
    let (fx, h) = world(|_| {});
    h.keep_for_a_day(&fx.path("agent-a1"));
    h.svc.hygiene_step("alpha");
    assert_eq!(state(&h, "agent-a1"), Some((HygieneState::Removing, None)));
    assert!(through_debounce(&h, &fx.path("agent-a1")));
}

#[test]
fn a_lock_waits_for_the_lead_but_brocks_own_lock_does_not() {
    let (fx, h) = world(|fx| {
        fx.git(&["worktree", "lock", "--reason", "owner: hands off", &fx.path("agent-a1")]);
    });
    assert_eq!(left_why(&h, "agent-a1"), "it is locked (owner: hands off)");
    assert!(!through_debounce(&h, &fx.path("agent-a1")));
    let (fx, h) = world(|fx| {
        fx.git(&[
            "worktree",
            "lock",
            "--reason",
            "cloned node_modules from X:/brock",
            &fx.path("agent-a1"),
        ]);
    });
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Removing));
    assert!(
        through_debounce(&h, &fx.path("agent-a1")),
        "the removal lifts Brock's lock"
    );
}

#[test]
fn a_main_checkout_is_never_the_hygienes() {
    let fx = Fixture::fake();
    let main_rule = Rule::folder("W-2", &fx.root, "worker").filtered(&["proj"], &[]);
    let h = Harness::new(&fx.root, vec![main_rule], hygiene_on);
    h.board().agents = vec![AgentFact {
        worktree: Some(fx.repo.clone()),
        claude_id: None,
        ..agent("1", "lead", "done", h.clock_now() - 1000)
    }];
    h.scan();
    h.svc.start();
    assert_eq!(state(&h, "proj"), None, "the hygiene never judges a main checkout");
    step(&h, DEBOUNCE * 100);
    h.svc.requests_idle();
    assert_eq!(h.svc.request_list("alpha").len(), 0);
    assert!(inbox(&h).is_empty());
}

#[test]
fn the_threads_own_release_answers_with_a_request_the_worker_removes_and_refuses_what_holds_work()
{
    let (fx, h) = world_with(&[("1", "lead", "working"), ("2", "lead", "working")], |_| {});
    let release = |name: &str| {
        let id = h.id_of(&fx.path(name));
        h.svc
            .release(&RemoveInput::new("alpha", &id, Caller::Thread("lead".into())))
    };
    h.board().agents[0].status = "done".into();
    h.scan();
    let queued = release("agent-a1").unwrap();
    assert_eq!(
        queued.text,
        format!(
            "{}: Argus removes it now; the result comes to your inbox (coord worktree requests shows it)",
            queued.id
        )
    );
    h.svc.requests_idle();
    let done = h.svc.request_list("alpha");
    assert!(done[0].release && done[0].branch, "{:?}", done[0]);
    assert_eq!(done[0].removed(), 1, "{:?}", done[0].results);
    assert!(!exists_itself(&fx.path("agent-a1")));
    assert_eq!(h.meta().hygiene.removed[0].path, fx.path("agent-a1"));
    let err = release("agent-a2").unwrap_err();
    assert!(
        err.message.contains("is not released: its agent A-2 is working"),
        "{}",
        err.message
    );
    h.board().agents[1].status = "done".into();
    put(&fx.path("agent-a2"), "wip.txt", "work\n");
    h.scan();
    let err = release("agent-a2").unwrap_err();
    assert!(
        err.message.contains("is not released: it has 1 uncommitted change")
            && err.message.contains("coord worktree approve"),
        "{}",
        err.message
    );
    assert!(exists_itself(&fx.path("agent-a2")));
    let page = h.svc.release(&RemoveInput::new(
        "alpha",
        &h.id_of(&fx.path("agent-a2")),
        Caller::Page,
    ));
    assert!(page.unwrap_err().message.contains("release is a thread's word"));
}

#[test]
fn the_threads_release_answers_at_once_while_a_removal_holds_the_turn() {
    let (fx, h) = world(|_| {});
    let id = h.id_of(&fx.path("agent-a1"));
    let turn = h.svc.one_at_a_time("alpha", "a test", None);
    let svc = h.svc.clone();
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let input = RemoveInput::new("alpha", &id, Caller::Thread("lead".into()));
        let _ = sender.send(svc.release(&input));
    });
    let answered = receiver.recv_timeout(std::time::Duration::from_secs(30));
    drop(turn);
    let queued = answered
        .expect("release answered while the turn was held")
        .unwrap();
    assert!(queued.text.contains("Argus removes it now"), "{}", queued.text);
    h.svc.requests_idle();
    assert!(!exists_itself(&fx.path("agent-a1")));
}

#[test]
fn a_junction_inside_is_removed_as_a_link_and_its_target_stays() {
    let fx = Fixture::fake();
    let path = fx.add("agent-a1");
    put(&fx.repo, ".git/info/exclude", "lnk\n");
    let sibling = join_path(&fx.root, "sibling");
    fill_sibling(Path::new(&sibling));
    make_link(&join_path(&path, "lnk"), &sibling);
    let h = Harness::new(&fx.root, vec![agents_rule(&fx)], |o| {
        hygiene_on(o);
        o.drop_links_first = true;
    });
    h.board().agents = vec![agent("1", "lead", "done", h.clock_now() - 1000)];
    h.scan();
    h.svc.start();
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Removing));
    assert!(through_debounce(&h, &path));
    assert_sibling_intact(Path::new(&sibling));
}

#[test]
fn a_restart_keeps_when_it_goes() {
    let (fx, h) = world(|_| {});
    step(&h, DEBOUNCE / 2);
    let goes = h.docs()["agent-a1"].hygiene.clone().unwrap().goes_at;
    let again = Harness::on_store(h.store.clone(), &fx.root, RemoveMode::Real, hygiene_on);
    again.clock.set(h.clock.now_ms());
    again.scan();
    assert_eq!(again.docs()["agent-a1"].hygiene.clone().unwrap().goes_at, goes);
}
