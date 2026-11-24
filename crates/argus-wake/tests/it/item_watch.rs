//! KEEP: K1.
//! The item watch: each export of a project board is compared with the one before and the events of the
//! table in `docs/wake-policy.md` become wakes, owner notices and activity lines, each once. The first export
//! only seeds, a replay after a restart sends nothing again, and a manual clock drives the quiet to-dos.

// test-tag: critical

use argus_core::fixture::with_items;
use argus_wake::{
    item_watch::{ItemWatch, WatchEnv, Watched},
    memory::MemoryBoards,
    reasons::{IMMEDIATE_REASONS, WAKE_REASONS},
    testkit::{HOUR, START, SentLookup, board, never_sent, sent_wakes},
    time::iso,
    wake::WakeInput,
    wake_policy::{WakeClass, class_of},
};
use serde_json::{Value, json};

fn item(id: &str, extra: &Value) -> Value {
    let mut base = json!({
        "id": id, "title": format!("Title of {id}"), "thread": "api", "status": "doing",
        "rev": 1, "updatedBy": "api",
    });
    for (key, value) in extra.as_object().into_iter().flatten() {
        base[key.as_str()] = value.clone();
    }
    base
}

fn alpha(items: &[Value], rest: &Value) -> Value {
    let base = json!({
        "threads": {
            "lead": {"id": "lead", "role": "lead", "status": "active"},
            "api": {"id": "api", "role": "thread", "status": "active"},
            "ui": {"id": "ui", "role": "thread", "status": "active"},
        },
        "items": {},
    });
    with_items(base, items, rest)
}

fn beta() -> Value {
    json!({"threads": {"blead": {"id": "blead", "role": "lead", "status": "active"}}})
}

struct World {
    boards: MemoryBoards,
    watch: ItemWatch,
    last_sent: SentLookup,
    wakes: Vec<WakeInput>,
    now: i64,
}

impl World {
    /// A world whose first export of `alpha` was `first`: the watch has seeded.
    fn new(first: &Value) -> Self {
        let boards = MemoryBoards::new();
        boards.set("beta", board(&beta()));
        let mut world = Self {
            boards,
            watch: ItemWatch::new(),
            last_sent: never_sent(),
            wakes: Vec::new(),
            now: START,
        };
        let seeded = world.export(first);
        assert_eq!(
            seeded.wakes.len() + seeded.owner.len(),
            0,
            "the first export only seeds"
        );
        world
    }

    fn export(&mut self, value: &Value) -> Watched {
        self.boards.set("alpha", board(value));
        let env = WatchEnv {
            boards: &self.boards,
            last_sent: &*self.last_sent,
            now: self.now,
        };
        let out = self.watch.on_board("alpha", &env);
        self.wakes.extend(out.wakes.clone());
        out
    }

    /// The process restarts: nothing in memory, the wakes table still holds every wake sent.
    fn restart(&mut self) {
        self.last_sent = sent_wakes(&self.wakes, self.now);
        self.watch = ItemWatch::new();
    }
}

fn lines(out: &Watched) -> Vec<(&str, &str, &str)> {
    out.wakes
        .iter()
        .map(|w| (w.scope.as_str(), w.thread.as_str(), w.reason.as_str()))
        .collect()
}

#[test]
fn a_hand_over_by_the_lead_tells_the_new_thread_and_leaves_a_notice_to_the_old_one() {
    let mut world = World::new(&alpha(&[item("I-1", &json!({}))], &json!({})));
    let moved = item(
        "I-1",
        &json!({"thread": "ui", "assignee": "ui", "updatedBy": "lead", "rev": 2}),
    );
    let out = world.export(&alpha(&[moved], &json!({})));
    assert_eq!(
        lines(&out),
        [
            ("alpha", "api", "item-unassigned"),
            ("alpha", "ui", "item-assigned")
        ]
    );
    assert_eq!(out.wakes[1].subject.as_deref(), Some("I-1:2:assigned"));
    assert_eq!(
        out.wakes[1].text,
        "I-1 (Title of I-1) was assigned to you by lead"
    );
}

#[test]
fn a_stage_change_by_someone_else_is_a_notice_to_the_assignee_and_by_the_assignee_is_nothing() {
    let mut world = World::new(&alpha(&[item("I-1", &json!({}))], &json!({})));
    let by_lead = item(
        "I-1",
        &json!({"status": "review", "updatedBy": "lead", "rev": 2}),
    );
    let out = world.export(&alpha(&[by_lead], &json!({})));
    assert_eq!(lines(&out), [("alpha", "api", "item-status")]);
    assert!(out.wakes[0].text.contains("doing to review"));
    let own = item(
        "I-1",
        &json!({"status": "done", "updatedBy": "api", "rev": 3}),
    );
    assert_eq!(world.export(&alpha(&[own], &json!({}))).wakes.len(), 0);
}

fn ask() -> Value {
    json!({"Q-1": {"id": "Q-1", "kind": "question", "status": "open", "item": "I-1",
        "author": "ui", "thread": "ui", "text": "Which way?", "rev": 1}})
}

#[test]
fn a_question_by_another_thread_on_your_item_is_a_notice_and_the_answer_unblocks_it() {
    let one = item("I-1", &json!({}));
    let mut world = World::new(&alpha(std::slice::from_ref(&one), &json!({"entries": {}})));
    let asked = world.export(&alpha(
        std::slice::from_ref(&one),
        &json!({"entries": ask()}),
    ));
    assert_eq!(lines(&asked), [("alpha", "api", "item-ask")]);
    let mut answered = ask();
    answered["Q-1"]["status"] = json!("answered");
    answered["Q-1"]["rev"] = json!(2);
    let out = world.export(&alpha(&[one], &json!({"entries": answered})));
    assert_eq!(lines(&out), [("alpha", "api", "item-unblocked")]);
    assert!(out.wakes[0].text.contains("Q-1 settled"));
}

#[test]
fn the_last_child_done_tells_the_parents_assignee_and_a_plan_run_ending_tells_the_plans_thread() {
    let epic = item("I-1", &json!({"type": "epic", "thread": "lead"}));
    let kid = |id: &str, status: &str, rev: i64| {
        item(id, &json!({"parent": "I-1", "status": status, "rev": rev}))
    };
    let mut world = World::new(&alpha(
        &[epic.clone(), kid("I-2", "done", 2), kid("I-3", "doing", 1)],
        &json!({}),
    ));
    let out = world.export(&alpha(
        &[epic, kid("I-2", "done", 2), kid("I-3", "done", 2)],
        &json!({}),
    ));
    assert_eq!(lines(&out), [("alpha", "lead", "children-done")]);
    assert!(out.wakes[0].text.contains("every child of I-1"));
    let run = |stage: &str, rev: i64| {
        item(
            "I-9",
            &json!({"type": "test-run", "stage": stage, "rev": rev, "n": 1, "fields": {
                "plan": "I-8", "results": [
                    {"test": "T-1", "status": "pass"}, {"test": "T-2", "status": "fail"}]}}),
        )
    };
    let mut world = World::new(&alpha(&[run("running", 2)], &json!({})));
    let out = world.export(&alpha(&[run("reported", 3)], &json!({})));
    assert_eq!(lines(&out), [("alpha", "api", "plan-run-done")]);
    assert!(out.wakes[0].text.ends_with("run 1: 1 pass, 1 fail"));
}

#[test]
fn a_to_do_goes_to_the_lead_of_another_project_to_a_thread_or_to_the_owner() {
    let todo = |assignee: &str| {
        json!({"H-1": {"id": "H-1", "text": "Expose the hook", "status": "pending",
            "item": "I-1", "assignee": assignee, "updatedBy": "api", "rev": 1}})
    };
    let base = alpha(&[item("I-1", &json!({}))], &json!({"actions": {}}));
    let sent = |assignee: &str| {
        let mut world = World::new(&base);
        let mut next = base.clone();
        next["actions"] = todo(assignee);
        world.export(&next)
    };
    assert_eq!(
        lines(&sent("lead:beta")),
        [("beta", "blead", "todo-assigned")]
    );
    assert_eq!(lines(&sent("ui")), [("alpha", "ui", "todo-assigned")]);
    let to_owner = sent("owner");
    assert_eq!(to_owner.wakes.len(), 0);
    assert_eq!(to_owner.owner.len(), 1);
    assert_eq!(to_owner.owner[0].topic, "assigned");
    assert_eq!(to_owner.owner[0].since, "H-1:1:todo-assigned");
}

#[test]
fn the_owner_is_noticed_of_an_assignment_a_bug_to_verify_and_a_plan_to_run() {
    let one = item("I-1", &json!({"thread": "lead"}));
    let mut world = World::new(&alpha(&[one], &json!({})));
    let given = item(
        "I-1",
        &json!({"thread": "lead", "assignee": "owner", "updatedBy": "lead", "rev": 2}),
    );
    let out = world.export(&alpha(&[given], &json!({})));
    assert_eq!(out.wakes.len(), 0);
    let notice = &out.owner[0];
    assert_eq!(
        (
            notice.topic.as_str(),
            notice.project.as_str(),
            notice.thread.as_str(),
            notice.since.as_str()
        ),
        ("assigned", "alpha", "lead", "I-1:2:assigned")
    );
    assert_eq!(
        notice.text,
        "alpha: I-1 (Title of I-1) was assigned to you by lead"
    );
    let bug = |stage: &str| {
        item(
            "I-2",
            &json!({"type": "bug", "stage": stage, "rev": 2, "createdBy": "human",
                "fields": {"reporter": "owner"}}),
        )
    };
    let mut world = World::new(&alpha(&[bug("fixing")], &json!({})));
    let out = world.export(&alpha(&[bug("verifying")], &json!({})));
    assert_eq!(
        out.owner
            .iter()
            .map(|n| n.topic.as_str())
            .collect::<Vec<_>>(),
        ["verify"]
    );
    let plan = |stage: &str| {
        item(
            "I-3",
            &json!({"type": "test-run", "stage": stage, "rev": 2, "fields": {"plan": "I-2", "build": "b1"}}),
        )
    };
    let builds = json!({"launchers": {"b1": {"name": "b1"}}});
    let mut world = World::new(&alpha(&[plan("draft")], &builds));
    let out = world.export(&alpha(&[plan("ready")], &builds));
    assert_eq!(
        out.owner
            .iter()
            .map(|n| n.topic.as_str())
            .collect::<Vec<_>>(),
        ["run"]
    );
}

#[test]
fn an_item_the_owner_holds_that_is_unblocked_is_a_line_of_the_activity_log_and_no_wake() {
    let held = |blocker: &Value| {
        item(
            "I-1",
            &json!({"thread": "lead", "assignee": "owner", "blocker": blocker}),
        )
    };
    let mut world = World::new(&alpha(&[held(&json!("legacy"))], &json!({})));
    let out = world.export(&alpha(&[held(&Value::Null)], &json!({})));
    assert_eq!(
        (out.wakes.len(), out.owner.len(), out.activity.len()),
        (0, 0, 1)
    );
    assert_eq!(out.activity[0].project.as_deref(), Some("alpha"));
}

#[test]
fn an_export_that_repeats_tells_nothing_and_a_replay_after_a_restart_tells_nothing_again() {
    let before = alpha(&[item("I-1", &json!({}))], &json!({}));
    let after = alpha(
        &[item(
            "I-1",
            &json!({"status": "review", "updatedBy": "lead", "rev": 2}),
        )],
        &json!({}),
    );
    let mut world = World::new(&before);
    assert_eq!(world.export(&after).wakes.len(), 1);
    assert_eq!(world.export(&after).wakes.len(), 0, "the same export again");
    world.restart();
    assert_eq!(
        world.export(&before).wakes.len(),
        0,
        "the first export seeds"
    );
    assert_eq!(world.export(&after).wakes.len(), 0, "the table has it");
    let newer = alpha(
        &[item(
            "I-1",
            &json!({"status": "done", "updatedBy": "lead", "rev": 3}),
        )],
        &json!({}),
    );
    assert_eq!(world.export(&newer).wakes.len(), 1, "news still comes");
}

#[test]
fn the_first_export_after_a_start_fires_nothing_for_what_was_already_there() {
    let owned = alpha(
        &[item(
            "I-1",
            &json!({"assignee": "owner", "updatedBy": "lead", "rev": 5}),
        )],
        &json!({}),
    );
    let _world = World::new(&owned);
}

#[test]
fn an_assignment_the_owners_post_woke_a_moment_ago_is_not_sent_twice() {
    let before = alpha(&[item("I-1", &json!({}))], &json!({}));
    let after = alpha(
        &[item(
            "I-1",
            &json!({"thread": "ui", "assignee": "ui", "updatedBy": "human", "rev": 2}),
        )],
        &json!({}),
    );
    let posted = |ago: i64| -> SentLookup {
        Box::new(move |scope, thread, reason, subject| {
            if (scope, thread, reason, subject) == ("alpha", "ui", "item-assigned", "I-1") {
                START - ago
            } else {
                0
            }
        })
    };
    let mut world = World::new(&before);
    world.last_sent = posted(10_000);
    assert_eq!(
        lines(&world.export(&after)),
        [("alpha", "api", "item-unassigned")]
    );
    let mut world = World::new(&before);
    world.last_sent = posted(5 * 60_000);
    assert_eq!(lines(&world.export(&after)).len(), 2);
}

#[test]
fn a_to_do_of_another_lead_that_holds_an_item_and_is_quiet_for_a_day_is_told_once() {
    let at = |ago: i64| iso(START - ago);
    let board_with = |updated: &str, assignee: &str| {
        alpha(
            &[item("I-3", &json!({"blockedBy": ["H-1"]}))],
            &json!({"actions": {"H-1": {"id": "H-1", "text": "Expose the hook", "status": "pending",
                "assignee": assignee, "createdAt": updated, "updatedAt": updated, "rev": 4}}}),
        )
    };
    let run = |world: &mut World| {
        let env = WatchEnv {
            boards: &world.boards,
            last_sent: &*world.last_sent,
            now: START,
        };
        world.watch.quiet(&env)
    };
    let mut world = World::new(&board_with(&at(25 * HOUR), "lead:beta"));
    let first = run(&mut world);
    assert_eq!(first.len(), 1);
    assert_eq!(
        (
            first[0].scope.as_str(),
            first[0].thread.as_str(),
            first[0].reason.as_str()
        ),
        ("beta", "blead", "todo-quiet")
    );
    assert_eq!(first[0].subject.as_deref(), Some("H-1:4"));
    assert_eq!(run(&mut world).len(), 0, "once");
    world.wakes.extend(first);
    world.restart();
    assert_eq!(run(&mut world).len(), 0, "and not again after a restart");
    let mut fresh = World::new(&board_with(&at(23 * HOUR), "lead:beta"));
    assert_eq!(run(&mut fresh).len(), 0, "not yet a day");
    let mut mine = World::new(&board_with(&at(25 * HOUR), "ui"));
    assert_eq!(run(&mut mine).len(), 0, "not a lead's");
}

#[test]
fn the_new_reasons_are_in_the_table_and_the_passive_ones_are_the_notices() {
    let passive = ["item-unassigned", "item-status", "item-ask", "todo-quiet"];
    let wakes = [
        "todo-assigned",
        "item-unblocked",
        "children-done",
        "plan-run-done",
    ];
    for reason in passive {
        assert!(WAKE_REASONS.contains(&reason), "{reason}");
        assert_eq!(class_of(reason), WakeClass::Passive, "{reason}");
    }
    for reason in wakes {
        assert!(WAKE_REASONS.contains(&reason), "{reason}");
        assert_eq!(class_of(reason), WakeClass::Wake, "{reason}");
    }
    for reason in passive.iter().chain(wakes.iter()) {
        assert!(!IMMEDIATE_REASONS.contains(reason), "{reason}");
    }
}
