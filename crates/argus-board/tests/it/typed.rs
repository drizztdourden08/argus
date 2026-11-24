//! Typed items in the board ops: the type, parent, assignee, stage, own fields and blockers of an
//! item, the flow and done rules of each type, `force`, retype, to-dos and notes for tests, and
//! the plain item that must behave as it always did.
//! I-70 P2a.

// test-tag: critical

use crate::common::{alpha, applied, doc, lead, refused};
use argus_board::ops::{OpResult, Role, Status, Who, apply_op, create_note};
use serde_json::{Value, json};

fn as_owner(board: &mut Value, op: &Value) -> OpResult {
    apply_op(board, op, &Who::human(), None)
}

fn worker() -> Who {
    Who {
        thread: Some("ui".to_owned()),
        role: Role::Thread,
        actor: None,
    }
}

fn make(board: &mut Value, who: &Who, set: Value) -> OpResult {
    let mut op = json!({"kind": "create", "target": {"c": "items"}});
    op["set"] = set;
    apply_op(board, &op, who, None)
}

fn made(board: &mut Value, who: &Who, set: Value) -> String {
    let res = make(board, who, set);
    applied(&res);
    res.extra["created"]["id"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

fn edit(board: &mut Value, who: &Who, id: &str, set: Value) -> OpResult {
    let mut op = json!({"kind": "edit", "target": {"c": "items", "id": id}});
    op["set"] = set;
    apply_op(board, &op, who, None)
}

fn edit_forced(board: &mut Value, who: &Who, id: &str, set: Value, force: &str) -> OpResult {
    let mut op = json!({"kind": "edit", "target": {"c": "items", "id": id}, "force": force});
    op["set"] = set;
    apply_op(board, &op, who, None)
}

fn typed(kind: &str, title: &str) -> Value {
    json!({"title": title, "thread": "ui", "type": kind})
}

fn bug_set(title: &str, parent: &str) -> Value {
    json!({"title": title, "thread": "ui", "type": "bug", "parent": parent,
        "fields": {"severity": "S2"}})
}

/// A board with a bug made by the lead and moved by the owner into `stage`.
fn bug_at(stage: &str) -> (Value, String) {
    let mut b = alpha();
    let epic = an_epic(&mut b);
    let id = made(&mut b, &lead(), bug_set("Crash", &epic));
    applied(&edit(&mut b, &Who::human(), &id, json!({"stage": stage})));
    (b, id)
}

/// A new epic for a story or a task to sit under; answers its id.
fn an_epic(b: &mut Value) -> String {
    made(b, &lead(), typed("epic", "Parent epic"))
}

/// An epic with one story under it; returns the epic id and the story id.
fn epic_with_story(b: &mut Value) -> (String, String) {
    let epic = made(b, &lead(), typed("epic", "Release"));
    let story = made(
        b,
        &lead(),
        json!({"title": "Story", "thread": "ui", "type": "story", "parent": epic}),
    );
    (epic, story)
}

fn forced_decisions(board: &Value) -> usize {
    board["entries"].as_object().map_or(0, |all| {
        all.values()
            .filter(|e| {
                e["kind"] == "decision"
                    && e["text"].as_str().is_some_and(|t| t.starts_with("Forced "))
            })
            .count()
    })
}

#[test]
fn an_item_without_a_type_is_written_and_changed_exactly_as_before() {
    let mut b = alpha();
    let id = "I-90".to_owned();
    b["items"][id.as_str()] = json!({
        "title": "Plain", "thread": "ui", "status": "todo", "origin": "lead",
        "ac": [{"n": 1, "text": "one", "done": false}],
    });
    let keys: Vec<String> = b["items"][id.as_str()]
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    for name in [
        "type",
        "parent",
        "assignee",
        "stage",
        "fields",
        "blockedBy",
        "images",
        "subject",
    ] {
        assert!(!keys.iter().any(|k| k == name), "{name} was written");
    }
    let entries = b["entries"].as_object().map_or(0, serde_json::Map::len);
    let done = edit(&mut b, &Who::human(), &id, json!({"status": "done"}));
    applied(&done);
    assert_eq!(done.outcome, format!("{id} edited (status)"));
    assert_eq!(done.extra.len(), 0);
    assert_eq!(b["items"][id.as_str()]["status"], "done");
    assert_eq!(b["items"][id.as_str()].get("stage"), None);
    assert_eq!(
        b["entries"].as_object().map_or(0, serde_json::Map::len),
        entries
    );
}

#[test]
fn the_subject_of_an_item_is_never_written_or_edited_again() {
    let mut b = alpha();
    let epic = an_epic(&mut b);
    let id = made(
        &mut b,
        &Who::human(),
        json!({"title": "No subject", "thread": "ui", "subject": "ui", "parent": epic}),
    );
    assert_eq!(b["items"][id.as_str()].get("subject"), None);
    let res = edit(&mut b, &Who::human(), "I-1", json!({"subject": "other"}));
    assert_eq!(refused(&res), "subject cannot be edited on items.");
    assert_eq!(b["items"]["I-1"]["subject"], "ui");
}

#[test]
fn a_bug_needs_its_severity_and_follows_its_flow() {
    let mut b = alpha();
    let epic = an_epic(&mut b);
    let missing = make(
        &mut b,
        &lead(),
        json!({"title": "Crash", "thread": "ui", "type": "bug", "parent": epic}),
    );
    assert!(refused(&missing).contains("severity"));
    let id = made(&mut b, &lead(), bug_set("Crash", &epic));
    assert_eq!(b["items"][id.as_str()]["type"], "bug");
    assert_eq!(b["items"][id.as_str()]["fields"]["severity"], "S2");
    let skip = edit(&mut b, &lead(), &id, json!({"status": "doing"}));
    let why = refused(&skip);
    assert!(why.contains("confirmed"), "{why}");
    applied(&edit(&mut b, &lead(), &id, json!({"stage": "confirmed"})));
    assert_eq!(b["items"][id.as_str()]["status"], "todo");
    applied(&edit(&mut b, &lead(), &id, json!({"status": "doing"})));
    assert_eq!(b["items"][id.as_str()]["stage"], "fixing");
    assert_eq!(b["items"][id.as_str()]["status"], "doing");
    let back = edit(&mut b, &lead(), &id, json!({"stage": "reported"}));
    assert!(refused(&back).contains("verifying"));
}

#[test]
fn the_owner_moves_a_bug_freely_but_never_past_the_done_rule() {
    let (mut b, id) = bug_at("fixing");
    assert_eq!(b["items"][id.as_str()]["status"], "doing");
    let closed = edit(&mut b, &Who::human(), &id, json!({"stage": "closed"}));
    assert!(refused(&closed).contains("no test case is linked"));
    b["tests"]["T-1"]["item"] = json!(id);
    applied(&edit(
        &mut b,
        &Who::human(),
        &id,
        json!({"stage": "closed"}),
    ));
    assert_eq!(b["items"][id.as_str()]["status"], "done");
}

#[test]
fn force_with_a_reason_overrides_a_refusing_rule_and_records_a_decision() {
    let (mut b, id) = bug_at("verifying");
    let flag = apply_op(
        &mut b,
        &json!({"kind": "edit", "target": {"c": "items", "id": id}, "set": {"stage": "closed"}, "force": true}),
        &Who::human(),
        None,
    );
    assert!(refused(&flag).contains("force with a reason"));
    assert_eq!(forced_decisions(&b), 0);
    let forced = edit_forced(
        &mut b,
        &Who::human(),
        &id,
        json!({"stage": "closed"}),
        "owner checked it by hand",
    );
    applied(&forced);
    assert_eq!(b["items"][id.as_str()]["stage"], "closed");
    assert_eq!(forced_decisions(&b), 1);
    let decision = b["entries"]
        .as_object()
        .and_then(|all| {
            all.values()
                .find(|e| e["kind"] == "decision" && e["item"] == json!(id))
        })
        .cloned()
        .unwrap_or(Value::Null);
    assert_eq!(
        decision["text"],
        format!("Forced {id} to closed: owner checked it by hand")
    );
    assert_eq!(decision["status"], "done");
    assert!(
        decision["context"]
            .as_str()
            .unwrap_or_default()
            .contains("no test case is linked")
    );
}

#[test]
fn an_epic_closes_only_when_every_child_is_done_or_dropped() {
    let mut b = alpha();
    let epic = made(&mut b, &lead(), typed("epic", "Release"));
    let child = made(
        &mut b,
        &lead(),
        json!({"title": "Part", "thread": "ui", "type": "task", "parent": epic}),
    );
    let res = edit(&mut b, &Who::human(), &epic, json!({"status": "done"}));
    assert!(refused(&res).contains(&child));
    applied(&edit(
        &mut b,
        &Who::human(),
        &child,
        json!({"status": "dropped"}),
    ));
    applied(&edit(
        &mut b,
        &Who::human(),
        &epic,
        json!({"status": "done"}),
    ));
    assert_eq!(b["items"][epic.as_str()]["stage"], "done");
}

#[test]
fn a_story_closes_only_with_its_criteria_checked() {
    let mut b = alpha();
    let epic = an_epic(&mut b);
    let story = made(
        &mut b,
        &lead(),
        json!({"title": "As a user", "thread": "ui", "type": "story", "parent": epic,
               "ac": ["it works"]}),
    );
    let res = edit(&mut b, &Who::human(), &story, json!({"status": "done"}));
    assert!(refused(&res).contains("AC1"));
    b["items"][story.as_str()]["ac"][0]["done"] = json!(true);
    applied(&edit(
        &mut b,
        &Who::human(),
        &story,
        json!({"status": "done"}),
    ));
}

#[test]
fn a_task_only_warns_about_open_criteria() {
    let mut b = alpha();
    let epic = an_epic(&mut b);
    let task = made(
        &mut b,
        &lead(),
        json!({"title": "Do it", "thread": "ui", "type": "task", "parent": epic, "ac": ["one"]}),
    );
    let res = edit(&mut b, &Who::human(), &task, json!({"status": "done"}));
    applied(&res);
    assert!(res.outcome.contains("Warning: criteria still open: AC1"));
    assert_eq!(res.extra["warnings"], json!(["criteria still open: AC1"]));
    assert_eq!(b["items"][task.as_str()]["status"], "done");
    assert_eq!(forced_decisions(&b), 0);
}

#[test]
fn a_move_op_follows_the_same_rules_as_an_edit() {
    let mut b = alpha();
    let epic = made(&mut b, &lead(), typed("epic", "Release"));
    made(
        &mut b,
        &lead(),
        json!({"title": "Part", "thread": "ui", "type": "task", "parent": epic}),
    );
    let drag =
        json!({"kind": "move", "target": {"c": "items", "id": epic}, "to": {"status": "done"}});
    assert!(refused(&apply_op(&mut b, &drag, &Who::human(), None)).contains("children not done"));
    let stage =
        json!({"kind": "move", "target": {"c": "items", "id": epic}, "to": {"stage": "active"}});
    applied(&apply_op(&mut b, &stage, &Who::human(), None));
    assert_eq!(b["items"][epic.as_str()]["status"], "doing");
    assert_eq!(b["items"][epic.as_str()]["stage"], "active");
}

#[test]
fn a_stage_and_a_status_that_disagree_are_refused() {
    let mut b = alpha();
    let epic = an_epic(&mut b);
    let id = made(&mut b, &lead(), bug_set("Crash", &epic));
    let res = edit(
        &mut b,
        &lead(),
        &id,
        json!({"stage": "fixing", "status": "todo"}),
    );
    assert!(refused(&res).contains("not in todo"));
    let unknown = edit(&mut b, &lead(), &id, json!({"stage": "nowhere"}));
    assert!(refused(&unknown).contains("has the stages"));
}

fn plan_set(title: &str, parent: &str) -> Value {
    json!({"title": title, "thread": "ui", "type": "test-plan", "parent": parent,
        "fields": {"cases": ["T-1"]}})
}

#[test]
fn a_test_plan_always_keeps_a_case() {
    let mut b = alpha();
    let epic = an_epic(&mut b);
    let bare = make(
        &mut b,
        &lead(),
        json!({"title": "Smoke", "thread": "ui", "type": "test-plan", "parent": epic}),
    );
    assert!(refused(&bare).contains("keeps at least one case"));
    let plan = made(&mut b, &lead(), plan_set("Smoke", &epic));
    let ready = edit(&mut b, &lead(), &plan, json!({"stage": "ready"}));
    applied(&ready);
    assert_eq!(b["items"][plan.as_str()]["fields"]["cases"], json!(["T-1"]));
    let emptied = edit(&mut b, &lead(), &plan, json!({"fields.cases": []}));
    assert!(refused(&emptied).contains("keeps at least one case"));
    assert_eq!(b["items"][plan.as_str()]["fields"]["cases"], json!(["T-1"]));
}

fn run_of(plan: &str, stage: &str, status: &str) -> Value {
    doc(
        "I-90",
        "ui",
        json!({
            "type": "test-run", "stage": stage, "status": "review", "n": 1,
            "endedAt": "2026-11-02T10:00:00Z",
            "fields": {"plan": plan, "results": [{"test": "T-1", "status": status}]},
        }),
        9000,
    )
}

#[test]
fn a_test_plan_passes_only_when_its_last_run_passed_every_case() {
    let mut b = alpha();
    let epic = an_epic(&mut b);
    let plan = made(&mut b, &lead(), plan_set("Smoke", &epic));
    let none = edit(&mut b, &Who::human(), &plan, json!({"stage": "passed"}));
    assert!(refused(&none).contains("no run yet"));
    b["items"]["I-90"] = run_of(&plan, "reported", "fail");
    let res = edit(&mut b, &Who::human(), &plan, json!({"stage": "passed"}));
    assert!(refused(&res).contains("cases not passed in run 1: T-1"));
    b["items"]["I-90"] = run_of(&plan, "passed", "pass");
    applied(&edit(
        &mut b,
        &Who::human(),
        &plan,
        json!({"stage": "passed"}),
    ));
}

#[test]
fn fields_are_checked_by_the_registry_and_belong_to_the_type() {
    let mut b = alpha();
    let epic = an_epic(&mut b);
    let bug = made(&mut b, &lead(), bug_set("Crash", &epic));
    let wrong = edit(&mut b, &lead(), &bug, json!({"fields.severity": "S9"}));
    assert!(refused(&wrong).contains("severity"));
    let foreign = edit(&mut b, &lead(), &bug, json!({"fields.goal": "ship"}));
    assert!(refused(&foreign).contains("not a field here"));
    let cleared = edit(&mut b, &lead(), &bug, json!({"fields.severity": null}));
    assert!(refused(&cleared).contains("required"));
    applied(&edit(
        &mut b,
        &lead(),
        &bug,
        json!({"fields.expected": "It stays up"}),
    ));
    assert_eq!(
        b["items"][bug.as_str()]["fields"]["expected"],
        "It stays up"
    );
    let epic = an_epic(&mut b);
    let task = made(
        &mut b,
        &lead(),
        json!({"title": "Plain task", "thread": "ui", "type": "task", "parent": epic}),
    );
    let none = edit(&mut b, &lead(), &task, json!({"fields.severity": "S1"}));
    assert!(refused(&none).contains("no fields of its own"));
}

#[test]
fn a_parent_must_be_allowed_acyclic_and_on_the_board() {
    let mut b = alpha();
    let (epic, story) = epic_with_story(&mut b);
    let task = made(
        &mut b,
        &lead(),
        json!({"title": "Leaf", "thread": "ui", "type": "task", "parent": story}),
    );
    assert_eq!(b["items"][task.as_str()]["parent"], json!(story));
    let root = edit(&mut b, &lead(), &epic, json!({"parent": story}));
    assert!(refused(&root).contains("cycle"));
    let own = edit(&mut b, &lead(), &story, json!({"parent": story}));
    assert!(refused(&own).contains("own parent"));
    let leaf_holds = edit(&mut b, &lead(), &story, json!({"parent": task}));
    assert!(refused(&leaf_holds).contains("cycle"));
    let wrong = make(
        &mut b,
        &lead(),
        json!({"title": "Odd", "thread": "ui", "type": "epic", "parent": epic}),
    );
    assert!(refused(&wrong).contains("always a root"));
    let under_task = make(
        &mut b,
        &lead(),
        json!({"title": "Odd", "thread": "ui", "type": "task", "parent": task}),
    );
    assert!(refused(&under_task).contains("a task sits under"));
    let ghost = edit(&mut b, &lead(), &task, json!({"parent": "I-99"}));
    assert_eq!(refused(&ghost), "no item \"I-99\".");
    let cleared = edit(&mut b, &lead(), &task, json!({"parent": "none"}));
    assert!(refused(&cleared).contains("needs a parent"));
    assert_eq!(b["items"][task.as_str()]["parent"], json!(story));
}

#[test]
fn a_cycle_in_the_parents_is_refused_even_where_the_types_would_allow_it() {
    let mut b = alpha();
    let epic = an_epic(&mut b);
    let under =
        |title: &str| json!({"title": title, "thread": "ui", "type": "story", "parent": epic});
    let first = made(&mut b, &lead(), under("One"));
    let second = made(&mut b, &lead(), under("Two"));
    b["items"][first.as_str()]["parent"] = json!(second);
    let res = edit(&mut b, &lead(), &second, json!({"parent": first}));
    assert!(refused(&res).contains("would be a cycle"));
}

#[test]
fn a_retype_keeps_the_lane_and_needs_a_parent_and_children_that_still_fit() {
    let mut b = alpha();
    let (_, story) = epic_with_story(&mut b);
    let leaf = made(
        &mut b,
        &lead(),
        json!({"title": "Leaf", "thread": "ui", "type": "task", "parent": story, "status": "doing"}),
    );
    let kids = edit(
        &mut b,
        &lead(),
        &story,
        json!({"type": "bug", "fields": {"severity": "S3"}}),
    );
    assert!(refused(&kids).contains(&leaf));
    let sev = edit(&mut b, &lead(), &leaf, json!({"type": "bug"}));
    assert!(refused(&sev).contains("needs severity"));
    applied(&edit(
        &mut b,
        &lead(),
        &leaf,
        json!({"type": "bug", "fields": {"severity": "S3"}}),
    ));
    assert_eq!(b["items"][leaf.as_str()]["type"], "bug");
    assert_eq!(b["items"][leaf.as_str()]["status"], "doing");
    assert_eq!(b["items"][leaf.as_str()]["stage"], "fixing");
    let orphan = edit(&mut b, &lead(), &leaf, json!({"type": "epic"}));
    assert!(refused(&orphan).contains("always a root"));
    assert_eq!(b["items"][leaf.as_str()]["type"], "bug");
}

#[test]
fn the_assignee_and_the_thread_move_together() {
    let mut b = alpha();
    applied(&edit(
        &mut b,
        &Who::human(),
        "I-1",
        json!({"assignee": "lead"}),
    ));
    assert_eq!(b["items"]["I-1"]["thread"], "lead");
    assert_eq!(b["items"]["I-1"]["assignee"], "lead");
    assert_eq!(b["tests"]["T-2"]["thread"], "lead");
    assert_eq!(b["entries"]["Q-1"]["thread"], "lead");
    applied(&edit(
        &mut b,
        &Who::human(),
        "I-1",
        json!({"assignee": "owner"}),
    ));
    assert_eq!(b["items"]["I-1"]["thread"], "lead");
    assert_eq!(b["items"]["I-1"]["assignee"], "owner");
    let lead_of_other = edit(
        &mut b,
        &Who::human(),
        "I-1",
        json!({"assignee": "lead:brock"}),
    );
    assert!(refused(&lead_of_other).contains("only for to-dos"));
    let nobody = edit(&mut b, &Who::human(), "I-1", json!({"assignee": "ghost"}));
    assert!(refused(&nobody).contains("not a thread of this board"));
}

#[test]
fn a_thread_cannot_hand_its_item_over_through_the_assignee() {
    let mut b = alpha();
    let res = edit(&mut b, &worker(), "I-1", json!({"assignee": "lead"}));
    assert!(refused(&res).contains("only the lead can hand it"));
    assert_eq!(b["items"]["I-1"]["thread"], "ui");
    applied(&edit(
        &mut b,
        &worker(),
        "I-1",
        json!({"assignee": "owner"}),
    ));
}

#[test]
fn a_new_item_assigned_to_a_thread_belongs_to_that_thread() {
    let mut b = alpha();
    let epic = an_epic(&mut b);
    let id = made(
        &mut b,
        &Who::human(),
        json!({"title": "Hand", "assignee": "ui", "parent": epic}),
    );
    assert_eq!(b["items"][id.as_str()]["thread"], "ui");
    let clash = make(
        &mut b,
        &Who::human(),
        json!({"title": "Clash", "thread": "lead", "assignee": "ui", "parent": epic}),
    );
    assert!(refused(&clash).contains("differ"));
}

#[test]
fn blockers_are_ids_of_the_board_and_never_close_a_cycle() {
    let mut b = alpha();
    applied(&edit(
        &mut b,
        &lead(),
        "I-2",
        json!({"blockedBy": ["I-1", "T-2", "H-1", "Q-1", "I-1"]}),
    ));
    assert_eq!(
        b["items"]["I-2"]["blockedBy"],
        json!(["I-1", "T-2", "H-1", "Q-1"])
    );
    let cycle = edit(&mut b, &lead(), "I-1", json!({"blockedBy": ["I-2"]}));
    assert!(refused(&cycle).contains("cycle"));
    let text = edit(
        &mut b,
        &lead(),
        "I-1",
        json!({"blockedBy": ["external: owner decides"]}),
    );
    assert!(refused(&text).contains("is not an id"));
    let myself = edit(&mut b, &lead(), "I-1", json!({"blockedBy": ["I-1"]}));
    assert!(refused(&myself).contains("cannot block itself"));
    let note = edit(&mut b, &lead(), "I-1", json!({"blockedBy": ["N-1"]}));
    assert!(refused(&note).contains("note never blocks"));
    let ghost = edit(&mut b, &lead(), "I-1", json!({"blockedBy": ["T-99"]}));
    assert!(refused(&ghost).contains("not on this board"));
    applied(&edit(&mut b, &lead(), "I-2", json!({"blockedBy": null})));
    assert_eq!(b["items"]["I-2"]["blockedBy"], json!([]));
}

#[test]
fn images_are_listed_with_their_proof_and_a_rich_text_may_name_them() {
    let mut b = alpha();
    let digest = "a".repeat(64);
    let pic = json!([{"n": 1, "file": "I-2-1.png", "caption": "The form", "sha256": digest}]);
    let epic = an_epic(&mut b);
    let id = made(
        &mut b,
        &lead(),
        json!({"title": "Pictured", "thread": "ui", "type": "task", "parent": epic, "images": pic,
               "description": "![The form](img:1)"}),
    );
    assert_eq!(b["items"][id.as_str()]["images"][0]["n"], 1);
    let ghost = edit(
        &mut b,
        &lead(),
        &id,
        json!({"description": "![Other](img:2)"}),
    );
    assert!(refused(&ghost).contains("image"));
    let bad = edit(
        &mut b,
        &lead(),
        &id,
        json!({"images": [{"n": 1, "file": "x.png", "caption": "x", "sha256": "short"}]}),
    );
    assert!(refused(&bad).contains("sha256"));
}

#[test]
fn a_typed_item_takes_its_text_from_the_registry_rules() {
    let mut b = alpha();
    let epic = an_epic(&mut b);
    let id = made(
        &mut b,
        &lead(),
        json!({"title": "Rich", "thread": "ui", "type": "task", "parent": epic}),
    );
    let lone = "This is a long paragraph without any bullet, written the old way, that goes on and on past the short limit of a single line, and then it keeps going for a while longer.";
    let res = edit(&mut b, &lead(), &id, json!({"description": lone}));
    assert!(refused(&res).contains("lone paragraph"));
    applied(&edit(
        &mut b,
        &lead(),
        &id,
        json!({"description": "Why\n- first\n- second"}),
    ));
}

#[test]
fn an_untyped_item_takes_its_description_by_the_rich_rule_too() {
    let mut b = alpha();
    let lone = "This is a long paragraph without any bullet, written the old way, that goes on and on past the short limit of a single line, and then it keeps going for a while longer.";
    let epic = an_epic(&mut b);
    let refused_new = make(
        &mut b,
        &lead(),
        json!({"title": "Plain", "thread": "ui", "parent": epic, "description": lone}),
    );
    assert!(refused(&refused_new).contains("lone paragraph"));
    let id = made(
        &mut b,
        &lead(),
        json!({"title": "Plain", "thread": "ui", "parent": epic, "description": "Why
- first"}),
    );
    let res = edit(&mut b, &lead(), &id, json!({"description": lone}));
    assert!(refused(&res).contains("lone paragraph"));
    assert_eq!(
        b["items"][id.as_str()]["description"],
        "Why
- first"
    );
}

#[test]
fn a_to_do_has_an_assignee_and_may_be_about_a_test() {
    let mut b = alpha();
    let ok = as_owner(
        &mut b,
        &json!({"kind": "create", "target": {"c": "actions"},
               "set": {"text": "Run the form test", "item": "T-2", "assignee": "lead:brock"}}),
    );
    applied(&ok);
    let id = ok.extra["created"]["id"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    assert_eq!(b["actions"][id.as_str()]["item"], "T-2");
    assert_eq!(b["actions"][id.as_str()]["thread"], "ui");
    assert_eq!(b["actions"][id.as_str()]["assignee"], "lead:brock");
    let ghost = as_owner(
        &mut b,
        &json!({"kind": "create", "target": {"c": "actions"},
               "set": {"text": "Who", "item": "I-1", "assignee": "lead:nobody"}}),
    );
    assert!(refused(&ghost).contains("to-dos"));
    let missing = as_owner(
        &mut b,
        &json!({"kind": "create", "target": {"c": "actions"}, "set": {"text": "Test", "item": "T-99"}}),
    );
    assert_eq!(refused(&missing), "no test \"T-99\".");
    let plain = as_owner(
        &mut b,
        &json!({"kind": "create", "target": {"c": "actions"}, "set": {"text": "Plain", "item": "I-1"}}),
    );
    applied(&plain);
    let plain_id = plain.extra["created"]["id"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    assert_eq!(b["actions"][plain_id.as_str()].get("assignee"), None);
    let edited = as_owner(
        &mut b,
        &json!({"kind": "edit", "target": {"c": "actions", "id": plain_id},
               "set": {"assignee": "owner", "item": "T-1"}}),
    );
    applied(&edited);
    assert_eq!(b["actions"][plain_id.as_str()]["item"], "T-1");
    assert_eq!(b["actions"][plain_id.as_str()]["assignee"], "owner");
}

#[test]
fn an_entry_moves_onto_a_test() {
    let mut b = alpha();
    let res = as_owner(
        &mut b,
        &json!({"kind": "move", "target": {"c": "entries", "id": "Q-1"}, "to": {"item": "T-2"}}),
    );
    applied(&res);
    assert_eq!(b["entries"]["Q-1"]["item"], "T-2");
    assert_eq!(b["entries"]["Q-1"]["thread"], "ui");
}

#[test]
fn a_note_is_an_entry_with_its_own_id_that_never_blocks() {
    let mut b = alpha();
    let id = create_note(&mut b, &lead(), "I-1", &json!("Looks fine so far")).unwrap_or_default();
    assert_eq!(id, "N-2");
    let note = &b["entries"]["N-2"];
    assert_eq!(note["kind"], "note");
    assert_eq!(note["item"], "I-1");
    assert_eq!(note["thread"], "ui");
    assert_eq!(note["status"], "done");
    let on_test = create_note(&mut b, &lead(), "T-2", &json!("Needs a device")).unwrap_or_default();
    assert_eq!(on_test, "N-3");
    assert_eq!(b["entries"]["N-3"]["item"], "T-2");
    assert_eq!(b["counters"]["N"], 3);
}

#[test]
fn a_note_follows_the_rich_text_rules_and_the_ownership_of_its_record() {
    let mut b = alpha();
    let lone = "A single paragraph that is much too long to be accepted as a short idea, because it has no bullet and no screenshot at all, so it must be refused.";
    let err = create_note(&mut b, &lead(), "I-1", &json!(lone))
        .unwrap_err()
        .to_string();
    assert!(err.contains("lone paragraph"), "{err}");
    let bullets = create_note(
        &mut b,
        &lead(),
        "I-1",
        &json!("Seen:\n- the form\n- the error"),
    );
    assert_eq!(bullets.unwrap_or_default(), "N-2");
    let missing = create_note(&mut b, &lead(), "I-99", &json!("x"))
        .unwrap_err()
        .to_string();
    assert_eq!(missing, "no item \"I-99\".");
    let blank = create_note(&mut b, &lead(), "I-1", &json!("   "))
        .unwrap_err()
        .to_string();
    assert_eq!(blank, "note text is required.");
    let mut other = alpha();
    other["items"]["I-2"]["thread"] = json!("lead");
    let foreign = create_note(&mut other, &worker(), "I-2", &json!("hello"))
        .unwrap_err()
        .to_string();
    assert!(
        foreign.contains("you cannot create records for lead"),
        "{foreign}"
    );
}

#[test]
fn a_force_reason_overrides_the_rule_but_keeps_the_revision_check() {
    let (mut b, id) = bug_at("verifying");
    applied(&edit(
        &mut b,
        &Who::human(),
        &id,
        json!({"title": "Crash again"}),
    ));
    let stale = |force: Value| {
        json!({
            "kind": "edit", "target": {"c": "items", "id": id}, "set": {"stage": "closed"},
            "baseRev": 1, "force": force,
        })
    };
    let kept = apply_op(
        &mut b,
        &stale(json!("checked by hand")),
        &Who::human(),
        None,
    );
    assert_eq!(kept.status, Status::Conflict, "{}", kept.outcome);
    assert_eq!(b["items"][id.as_str()]["stage"], "verifying");
    let skipped = apply_op(&mut b, &stale(json!(true)), &Who::human(), None);
    assert_ne!(skipped.status, Status::Conflict, "{}", skipped.outcome);
}

#[test]
fn a_story_or_task_without_an_epic_is_refused_and_keeps_its_parent_on_edit() {
    let mut b = alpha();
    let epic = made(&mut b, &lead(), typed("epic", "Open epic"));
    for kind in ["story", "task"] {
        let bare = make(&mut b, &lead(), typed(kind, "Orphan"));
        let text = refused(&bare);
        assert!(text.contains("--parent I-n"), "{text}");
        assert!(text.contains(&format!("{epic} \"Open epic\"")), "{text}");
    }
    let plain = make(&mut b, &lead(), json!({"title": "Bare", "thread": "ui"}));
    assert!(refused(&plain).contains("--parent I-n"));
    let child = made(
        &mut b,
        &lead(),
        json!({"title": "Part", "thread": "ui", "type": "task", "parent": epic}),
    );
    let clear = edit(&mut b, &lead(), &child, json!({"parent": "none"}));
    assert!(refused(&clear).contains("needs a parent"));
    let bug = made(&mut b, &lead(), bug_set("Parented", &epic));
    let cleared = edit(&mut b, &lead(), &bug, json!({"parent": "none"}));
    assert!(refused(&cleared).contains("needs a parent"));
}

#[test]
fn a_bug_without_a_parent_is_refused_and_one_under_an_epic_passes() {
    let mut b = alpha();
    let epic = an_epic(&mut b);
    for kind in ["bug", "debt", "test-plan"] {
        let bare = make(&mut b, &lead(), typed(kind, "Orphan"));
        let text = refused(&bare);
        assert!(text.contains("needs a parent"), "{text}");
        assert!(text.contains("--parent I-n"), "{text}");
    }
    let under = make(&mut b, &lead(), bug_set("Crash", &epic));
    applied(&under);
}

#[test]
fn an_old_bug_without_a_parent_stays_editable() {
    let mut b = alpha();
    let epic = an_epic(&mut b);
    let bug = made(&mut b, &lead(), bug_set("Old crash", &epic));
    b["items"][bug.as_str()]["parent"] = Value::Null;
    applied(&edit(
        &mut b,
        &lead(),
        &bug,
        json!({"title": "Old crash, renamed"}),
    ));
    assert_eq!(b["items"][bug.as_str()]["title"], "Old crash, renamed");
}
