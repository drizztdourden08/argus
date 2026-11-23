// test-tag: critical

use argus_core::fixture::with_items;
use serde_json::{Value, json};

use super::{EventKind, Hearer, ItemEvent, item_events};

fn item(id: &str, extra: &Value) -> Value {
    let mut base =
        json!({"id": id, "title": format!("Title of {id}"), "status": "doing", "rev": 1});
    for (key, value) in extra.as_object().into_iter().flatten() {
        base[key.as_str()] = value.clone();
    }
    base
}

fn board(items: &[Value], rest: &Value) -> Value {
    with_items(json!({"items": {}}), items, rest)
}

fn kinds(events: &[ItemEvent]) -> Vec<&'static str> {
    events.iter().map(|e| e.kind.key()).collect()
}

fn only(events: Vec<ItemEvent>) -> ItemEvent {
    assert_eq!(events.len(), 1, "{events:?}");
    events.into_iter().next().expect("one event")
}

fn thread(name: &str) -> Hearer {
    Hearer::Thread(name.to_owned())
}

#[test]
fn a_board_that_did_not_change_has_no_events() {
    let b = board(&[item("I-1", &json!({"thread": "api"}))], &json!({}));
    assert_eq!(item_events(&b, &b).len(), 0);
}

#[test]
fn an_item_given_to_another_thread_is_told_to_it_and_taken_from_the_old_one() {
    let before = board(&[item("I-1", &json!({"thread": "api"}))], &json!({}));
    let after = board(
        &[item(
            "I-1",
            &json!({"thread": "ui", "assignee": "ui", "updatedBy": "lead", "rev": 2}),
        )],
        &json!({}),
    );
    let events = item_events(&before, &after);
    assert_eq!(kinds(&events), ["unassigned", "assigned"]);
    assert_eq!(events[0].to, thread("api"));
    assert_eq!(events[1].to, thread("ui"));
    assert_eq!(events[1].subject(), "I-1:2:assigned");
}

#[test]
fn a_thread_that_hands_off_or_takes_its_own_item_says_nothing_to_itself() {
    let before = board(&[item("I-1", &json!({"thread": "api"}))], &json!({}));
    let handed = board(
        &[item(
            "I-1",
            &json!({"thread": "ui", "updatedBy": "api", "rev": 2}),
        )],
        &json!({}),
    );
    assert_eq!(kinds(&item_events(&before, &handed)), ["assigned"]);
    let taken = board(
        &[item(
            "I-1",
            &json!({"thread": "ui", "updatedBy": "ui", "rev": 2}),
        )],
        &json!({}),
    );
    assert_eq!(kinds(&item_events(&before, &taken)), ["unassigned"]);
}

#[test]
fn a_new_item_for_another_thread_is_an_assignment() {
    let before = board(&[], &json!({}));
    let after = board(
        &[item("I-2", &json!({"thread": "ui", "updatedBy": "lead"}))],
        &json!({}),
    );
    let event = only(item_events(&before, &after));
    assert_eq!(
        (event.kind, event.to.clone()),
        (EventKind::Assigned, thread("ui"))
    );
    assert_eq!(event.by, "lead");
}

#[test]
fn an_item_given_to_the_owner_by_a_thread_is_told_to_him_but_not_when_he_did_it() {
    let before = board(&[item("I-1", &json!({"thread": "lead"}))], &json!({}));
    let given = |by: &str| {
        board(
            &[item(
                "I-1",
                &json!({"thread": "lead", "assignee": "owner", "updatedBy": by, "rev": 2}),
            )],
            &json!({}),
        )
    };
    let by_lead = item_events(&before, &given("lead"));
    assert_eq!(kinds(&by_lead), ["assigned"]);
    assert_eq!(by_lead[0].to, Hearer::Owner);
    assert_eq!(by_lead[0].by, "lead");
    assert_eq!(item_events(&before, &given("human")).len(), 0);
}

#[test]
fn an_item_given_to_an_agent_wakes_its_thread_and_never_a_thread_called_agent() {
    let before = board(&[item("I-1", &json!({"thread": "api"}))], &json!({}));
    let agents = json!({"agents": {"A-1": {"id": "A-1", "thread": "ui"}}});
    let given = |by: &str| {
        let held = json!({"thread": "api", "assignee": "agent:A-1", "updatedBy": by, "rev": 2});
        board(&[item("I-1", &held)], &agents)
    };
    let heard =
        |events: &[ItemEvent], who: &str| events.iter().filter(|e| e.to == thread(who)).count();
    let by_lead = item_events(&before, &given("lead"));
    assert_eq!(heard(&by_lead, "agent:A-1"), 0);
    assert_eq!(heard(&by_lead, "ui"), 1);
    let by_agent_thread = item_events(&before, &given("ui"));
    assert_eq!(heard(&by_agent_thread, "agent:A-1"), 0);
    assert_eq!(heard(&by_agent_thread, "ui"), 0);
}

/// A board with the to-do `H-1` on `I-1`, its fields overridden by `extra`.
fn todo_board(extra: &Value) -> Value {
    let mut t = json!({"id": "H-1", "text": "Check the log", "status": "pending", "item": "I-1", "rev": 1, "updatedBy": "api"});
    for (key, value) in extra.as_object().into_iter().flatten() {
        t[key.as_str()] = value.clone();
    }
    board(&[], &json!({"actions": {"H-1": t}}))
}

#[test]
fn a_to_do_is_told_to_the_owner_a_lead_or_a_thread_it_names_and_to_nobody_it_does_not() {
    let before = board(&[], &json!({}));
    let to = |extra: Value| item_events(&before, &todo_board(&extra));
    assert_eq!(to(json!({"assignee": "owner"}))[0].to, Hearer::Owner);
    assert_eq!(
        to(json!({"assignee": "lead:tessera"}))[0].to,
        Hearer::Lead("tessera".to_owned())
    );
    assert_eq!(to(json!({"assignee": "ui"}))[0].to, thread("ui"));
    assert_eq!(to(json!({})).len(), 0);
    assert_eq!(to(json!({"assignee": "api"})).len(), 0);
    assert_eq!(to(json!({"assignee": "owner", "status": "done"})).len(), 0);
    let event = &to(json!({"assignee": "owner"}))[0];
    assert_eq!(
        (event.kind, event.item.as_str(), event.title.as_str()),
        (EventKind::TodoAssigned, "I-1", "Check the log")
    );
}

#[test]
fn a_to_do_that_keeps_its_assignee_is_told_once() {
    let one = json!({"id": "H-1", "text": "x", "status": "pending", "assignee": "owner", "rev": 1});
    let b = board(&[], &json!({"actions": {"H-1": one}}));
    assert_eq!(item_events(&b, &b).len(), 0);
}

#[test]
fn a_stage_change_by_someone_else_is_told_to_the_assignee_thread() {
    let before = board(
        &[item("I-1", &json!({"thread": "api", "status": "doing"}))],
        &json!({}),
    );
    let after = |by: &str| {
        board(
            &[item(
                "I-1",
                &json!({"thread": "api", "status": "review", "updatedBy": by, "rev": 2}),
            )],
            &json!({}),
        )
    };
    let event = only(item_events(&before, &after("human")));
    assert_eq!(
        (event.kind, event.to.clone(), event.detail.as_str()),
        (EventKind::StageChanged, thread("api"), "doing to review")
    );
    assert_eq!(item_events(&before, &after("api")).len(), 0);
}

#[test]
fn a_stage_change_under_an_item_assigned_to_the_owner_goes_to_the_accountable_thread() {
    let owned = |status: &str, by: &str, rev: i64| {
        item(
            "I-1",
            &json!({"thread": "lead", "assignee": "owner", "status": status, "updatedBy": by, "rev": rev}),
        )
    };
    let before = board(&[owned("doing", "lead", 1)], &json!({}));
    let after = board(&[owned("review", "human", 2)], &json!({}));
    let event = only(item_events(&before, &after));
    assert_eq!(
        (event.kind, event.to.clone()),
        (EventKind::StageChanged, thread("lead"))
    );
}

#[test]
fn a_move_to_another_thread_is_an_assignment_and_not_also_a_status_change() {
    let before = board(
        &[item("I-1", &json!({"thread": "api", "status": "todo"}))],
        &json!({}),
    );
    let after = board(
        &[item(
            "I-1",
            &json!({"thread": "ui", "status": "doing", "updatedBy": "lead", "rev": 2}),
        )],
        &json!({}),
    );
    assert_eq!(
        kinds(&item_events(&before, &after)),
        ["unassigned", "assigned"]
    );
}

fn ask(id: &str, kind: &str, status: &str, by: &str) -> Value {
    json!({"id": id, "kind": kind, "status": status, "item": "I-1", "author": by, "thread": by, "text": "Which way?", "rev": 1})
}

#[test]
fn a_new_ask_by_another_thread_is_told_to_the_thread_of_the_item() {
    let items = [item("I-1", &json!({"thread": "api"}))];
    let before = board(&items, &json!({"entries": {}}));
    let after = |entry: Value| board(&items, &json!({"entries": {"Q-1": entry}}));
    let event = only(item_events(
        &before,
        &after(ask("Q-1", "question", "open", "ui")),
    ));
    assert_eq!(
        (
            event.kind,
            event.to.clone(),
            event.item.as_str(),
            event.detail.as_str()
        ),
        (EventKind::Ask, thread("api"), "I-1", "Which way?")
    );
    assert_eq!(event.subject(), "Q-1:1:ask");
    let decision = item_events(&before, &after(ask("Q-1", "decision", "pending", "ui")));
    assert_eq!(kinds(&decision), ["ask"]);
}

#[test]
fn an_ask_by_the_owner_by_the_thread_itself_an_answered_one_or_an_old_one_is_not_told() {
    let items = [item("I-1", &json!({"thread": "api"}))];
    let before = board(&items, &json!({"entries": {}}));
    let after = |entry: Value| board(&items, &json!({"entries": {"Q-1": entry}}));
    for entry in [
        ask("Q-1", "question", "open", "human"),
        ask("Q-1", "question", "open", "api"),
        ask("Q-1", "question", "answered", "ui"),
        ask("Q-1", "followup", "open", "ui"),
    ] {
        assert_eq!(item_events(&before, &after(entry)).len(), 0);
    }
    let old = after(ask("Q-1", "question", "open", "ui"));
    assert_eq!(item_events(&old, &old).len(), 0);
}

#[test]
fn the_last_blocker_settling_unblocks_the_item_for_its_assignee() {
    let blocked = item(
        "I-1",
        &json!({"thread": "api", "blockedBy": ["I-2", "Q-1"]}),
    );
    let before = board(
        &[blocked.clone(), item("I-2", &json!({"status": "doing"}))],
        &json!({"entries": {"Q-1": ask("Q-1", "question", "open", "ui")}}),
    );
    let one_left = board(
        &[blocked.clone(), item("I-2", &json!({"status": "done"}))],
        &json!({"entries": {"Q-1": ask("Q-1", "question", "open", "ui")}}),
    );
    assert_eq!(item_events(&before, &one_left).len(), 0);
    let mut answered = ask("Q-1", "question", "answered", "ui");
    answered["rev"] = json!(4);
    let none_left = board(
        &[blocked, item("I-2", &json!({"status": "done"}))],
        &json!({"entries": {"Q-1": answered}}),
    );
    let event = only(item_events(&one_left, &none_left));
    assert_eq!(
        (
            event.kind,
            event.to.clone(),
            event.detail.as_str(),
            event.rev
        ),
        (EventKind::Unblocked, thread("api"), "Q-1", 4)
    );
}

#[test]
fn an_ask_the_owner_answered_to_the_hearers_own_thread_is_told_by_the_answer_not_as_unblocked() {
    let blocked = item("I-1", &json!({"thread": "api"}));
    let settle = |asker: &str, status: &str| {
        let open = board(
            std::slice::from_ref(&blocked),
            &json!({"entries": {"Q-1": ask("Q-1", "question", "open", asker)}}),
        );
        let closed = board(
            std::slice::from_ref(&blocked),
            &json!({"entries": {"Q-1": ask("Q-1", "question", status, asker)}}),
        );
        item_events(&open, &closed)
    };
    assert_eq!(settle("api", "answered").len(), 0);
    assert_eq!(kinds(&settle("ui", "answered")), ["unblocked"]);
    assert_eq!(kinds(&settle("api", "dropped")), ["unblocked"]);
}

#[test]
fn an_answered_ask_next_to_another_cause_that_settled_is_still_unblocked() {
    let blocked = item("I-1", &json!({"thread": "api", "blockedBy": ["I-2"]}));
    let entries = |status: &str| json!({"entries": {"Q-1": ask("Q-1", "question", status, "api")}});
    let before = board(
        &[blocked.clone(), item("I-2", &json!({"status": "doing"}))],
        &entries("open"),
    );
    let after = board(
        &[blocked, item("I-2", &json!({"status": "done"}))],
        &entries("answered"),
    );
    assert_eq!(kinds(&item_events(&before, &after)), ["unblocked"]);
}

#[test]
fn an_item_for_the_owner_is_unblocked_to_him_and_a_closed_item_to_nobody() {
    let waiting = |blocker: &Value, status: &str| {
        item(
            "I-1",
            &json!({"assignee": "owner", "thread": "lead", "blocker": blocker, "status": status}),
        )
    };
    let before = board(&[waiting(&json!("legacy"), "doing")], &json!({}));
    let freed = board(&[waiting(&Value::Null, "doing")], &json!({}));
    let events = item_events(&before, &freed);
    assert_eq!(
        (kinds(&events), events[0].to.clone()),
        (vec!["unblocked"], Hearer::Owner)
    );
    let done = board(&[waiting(&Value::Null, "done")], &json!({}));
    assert_eq!(kinds(&item_events(&before, &done)), ["status"]);
}

#[test]
fn a_dependency_that_landed_is_an_event_with_its_id() {
    let waits = |state: &str| {
        item(
            "I-1",
            &json!({"thread": "api", "waitsOn": [{"dep": "D-3", "state": state}]}),
        )
    };
    let before = board(&[waits("open")], &json!({}));
    let after = board(&[waits("done")], &json!({}));
    let events = item_events(&before, &after);
    assert_eq!(kinds(&events), ["unblocked", "dep-landed"]);
    assert_eq!(events[1].detail, "D-3");
    assert_eq!(item_events(&after, &after).len(), 0);
}

fn tree(states: &[&str]) -> Value {
    let mut items = vec![item("I-1", &json!({"type": "epic", "thread": "lead"}))];
    for (n, status) in states.iter().enumerate() {
        let id = format!("I-{}", n + 2);
        items.push(item(
            &id,
            &json!({"parent": "I-1", "status": status, "rev": n + 2}),
        ));
    }
    board(&items, &json!({}))
}

#[test]
fn the_last_child_closing_tells_the_parents_assignee_once() {
    let events = item_events(&tree(&["done", "doing"]), &tree(&["done", "dropped"]));
    let event = only(events);
    assert_eq!(
        (
            event.kind,
            event.id.as_str(),
            event.to.clone(),
            event.detail.as_str()
        ),
        (EventKind::ChildrenDone, "I-1", thread("lead"), "2")
    );
    assert_eq!(event.rev, 3);
}

#[test]
fn a_child_closing_while_another_is_open_or_a_parent_already_closed_tells_nobody() {
    assert_eq!(
        item_events(&tree(&["doing", "doing"]), &tree(&["done", "doing"])).len(),
        0
    );
    assert_eq!(
        item_events(&tree(&["done", "done"]), &tree(&["done", "done"])).len(),
        0
    );
}

fn bug(stage: &str, status: &str, by: &str) -> Value {
    item(
        "I-1",
        &json!({"type": "bug", "thread": "api", "stage": stage, "status": status,
            "createdBy": "human", "updatedBy": by, "rev": 2, "fields": {"reporter": "owner"}}),
    )
}

#[test]
fn a_bug_the_owner_reported_reaching_verifying_is_told_to_him() {
    let before = board(&[bug("fixing", "doing", "api")], &json!({}));
    let after = board(&[bug("verifying", "review", "api")], &json!({}));
    let events = item_events(&before, &after);
    assert_eq!(kinds(&events), ["verify"]);
    assert_eq!(events[0].to, Hearer::Owner);
    let by_owner = board(&[bug("verifying", "review", "human")], &json!({}));
    assert_eq!(kinds(&item_events(&before, &by_owner)), ["status"]);
}

#[test]
fn a_bug_someone_else_reported_is_not_the_owners_to_verify() {
    let mut thread_bug = bug("verifying", "review", "api");
    thread_bug["fields"]["reporter"] = json!("ui");
    thread_bug["createdBy"] = json!("ui");
    let before = board(&[bug("fixing", "doing", "api")], &json!({}));
    let after = board(&[thread_bug], &json!({}));
    assert_eq!(item_events(&before, &after).len(), 0);
}

fn run(stage: &str, build: &str, results: &Value) -> Value {
    item(
        "I-9",
        &json!({"type": "test-run", "thread": "qa", "stage": stage, "updatedBy": "qa", "rev": 3,
            "n": 3, "fields": {"plan": "I-8", "build": build, "results": results}}),
    )
}

fn launchers(retired: bool) -> Value {
    let launcher = if retired {
        json!({"retiredAt": "2026-11-04T10:00:00Z"})
    } else {
        json!({"name": "build-1"})
    };
    json!({"launchers": {"build-1": launcher}})
}

#[test]
fn a_run_that_is_ready_with_a_live_build_is_told_to_the_owner_once() {
    let none = json!([]);
    let before = board(&[run("draft", "build-1", &none)], &launchers(false));
    let after = board(&[run("ready", "build-1", &none)], &launchers(false));
    let event = only(item_events(&before, &after));
    assert_eq!(
        (event.kind, event.to.clone(), event.detail.as_str()),
        (EventKind::RunReady, Hearer::Owner, "build-1")
    );
    assert_eq!(item_events(&after, &after).len(), 0);
}

#[test]
fn a_run_waits_for_its_build_and_is_told_when_the_build_turns_live() {
    let none = json!([]);
    let retired = board(&[run("ready", "build-1", &none)], &launchers(true));
    let before = board(&[run("draft", "build-1", &none)], &launchers(true));
    assert_eq!(item_events(&before, &retired).len(), 0);
    let live = board(&[run("ready", "build-1", &none)], &launchers(false));
    assert_eq!(kinds(&item_events(&retired, &live)), ["run-ready"]);
}

#[test]
fn a_run_handed_to_the_owner_is_left_to_the_handed_ping() {
    let none = json!([]);
    let mut handed = run("ready", "build-1", &none);
    handed["assignee"] = json!("owner");
    let before = board(&[run("draft", "build-1", &none)], &launchers(false));
    let after = board(&[handed.clone()], &launchers(false));
    assert_eq!(item_events(&before, &after).len(), 0);
    let retired = board(&[handed], &launchers(true));
    assert_eq!(kinds(&item_events(&before, &retired)), ["assigned"]);
}

#[test]
fn a_run_that_ended_is_told_to_its_thread_with_its_counts() {
    let open = json!([{"test": "T-1", "status": "pass"}]);
    let ended = json!([
        {"test": "T-1", "status": "pass"}, {"test": "T-2", "status": "pass"},
        {"test": "T-3", "status": "fail"}, {"test": "T-4", "status": "skip"}]);
    let before = board(&[run("running", "build-1", &open)], &launchers(false));
    let after = board(&[run("reported", "build-1", &ended)], &launchers(false));
    let events = item_events(&before, &after);
    let event = events
        .iter()
        .find(|e| e.kind == EventKind::PlanRunDone)
        .expect("run done");
    assert_eq!(event.to, thread("qa"));
    assert_eq!(event.detail, "run 3: 2 pass, 1 fail, 1 skip");
    assert_eq!(item_events(&after, &after).len(), 0);
}
