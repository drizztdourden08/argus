//! I-70 P3a: `coord item add|set|status|tree|list` with types, parents, assignees, stages and
//! `--force`, `coord types`, and the plain calls that must stay as they were.

// test-tag: critical

use crate::work_board::Work;
use serde_json::{Value, json};

fn item(work: &Work, id: &str) -> Value {
    work.board()["items"][id].clone()
}

fn decisions_about(work: &Work, id: &str) -> Vec<Value> {
    let board = work.board();
    let all = board["entries"].as_object().cloned().unwrap_or_default();
    all.values()
        .filter(|e| e["kind"] == "decision" && e["item"] == json!(id))
        .cloned()
        .collect()
}

#[test]
fn a_plain_add_prints_as_before_and_writes_no_subject_and_no_typed_field() {
    let work = Work::new();
    let said = work.worker_ok(&["item", "add", "Plain", "--ac", "one", "--ac", "two"]);
    assert_eq!(said, "I-2: Plain (worker, 2 AC)\n");
    let made = item(&work, "I-2");
    assert_eq!(made["status"], "todo");
    assert_eq!(made["thread"], "worker");
    for key in ["subject", "type", "assignee", "stage", "fields", "parent"] {
        assert!(made.get(key).is_none(), "{key} was written: {made}");
    }
}

#[test]
fn the_subject_flag_is_ignored_by_add_and_set() {
    let work = Work::new();
    work.worker_ok(&["item", "add", "Plain", "--subject", "ui"]);
    assert!(item(&work, "I-2").get("subject").is_none());
    let ran = work.worker(&["item", "set", "I-2", "--subject", "ui"]);
    assert_eq!(ran.code, 1);
    assert!(
        ran.err.contains(
            "nothing to change: pass --title, --desc, --status, --type, --parent, --assignee, --stage or --field."
        ),
        "{}",
        ran.err
    );
}

#[test]
fn add_takes_a_type_its_fields_and_a_named_flag_for_each_field() {
    let work = Work::new();
    let said = work.worker_ok(&[
        "item",
        "add",
        "Crash",
        "--type",
        "bug",
        "--field",
        "severity=S2",
    ]);
    assert_eq!(said, "I-2: Crash (worker, 0 AC)\n");
    let bug = item(&work, "I-2");
    assert_eq!(bug["type"], "bug");
    assert_eq!(bug["fields"]["severity"], "S2");
    work.worker_ok(&[
        "item",
        "add",
        "Hang",
        "--type",
        "bug",
        "--severity",
        "S1",
        "--expected",
        "It opens",
    ]);
    let hang = item(&work, "I-3");
    assert_eq!(hang["fields"]["severity"], "S1");
    assert_eq!(hang["fields"]["expected"], "It opens");
}

#[test]
fn add_refuses_what_the_registry_refuses() {
    let work = Work::new();
    let no_severity = work.worker(&["item", "add", "Crash", "--type", "bug"]);
    assert_eq!(no_severity.code, 1);
    assert!(
        no_severity.err.contains("needs severity"),
        "{}",
        no_severity.err
    );
    let bad_type = work.worker(&["item", "add", "Odd", "--type", "saga"]);
    assert!(
        bad_type.err.contains("type must be one of"),
        "{}",
        bad_type.err
    );
    let bad_field = work.worker(&["item", "add", "Odd", "--type", "task", "--field", "mood=ok"]);
    assert!(
        bad_field.err.contains("mood is not a field here"),
        "{}",
        bad_field.err
    );
    let bare = work.worker(&["item", "add", "Odd", "--type", "task", "--field", "mood"]);
    assert!(bare.err.contains("--field takes key=value"), "{}", bare.err);
    assert_eq!(
        work.board()["items"]
            .as_object()
            .map_or(0, serde_json::Map::len),
        1
    );
}

#[test]
fn add_places_a_child_under_its_parent_and_a_typed_add_for_a_thread_assigns_it() {
    let work = Work::new();
    work.worker_ok(&["item", "add", "Release", "--type", "epic"]);
    work.worker_ok(&["item", "add", "Story", "--type", "story", "--parent", "I-2"]);
    assert_eq!(item(&work, "I-3")["parent"], "I-2");
    let refused = work.worker(&[
        "item",
        "add",
        "Epic in epic",
        "--type",
        "epic",
        "--parent",
        "I-2",
    ]);
    assert!(
        refused.err.contains("a epic is always a root"),
        "{}",
        refused.err
    );
    let said = work.lead(&[
        "item",
        "add",
        "For the worker",
        "--type",
        "task",
        "--for",
        "worker",
    ]);
    assert_eq!(said.code, 0, "{}", said.err);
    let made = item(&work, "I-4");
    assert_eq!(made["thread"], "worker");
    assert_eq!(made["assignee"], "worker");
}

#[test]
fn set_retypes_assigns_and_clears_a_parent() {
    let work = Work::new();
    let said = work.worker_ok(&[
        "item",
        "set",
        "I-1",
        "--type",
        "bug",
        "--field",
        "severity=S3",
    ]);
    assert!(
        said.starts_with("I-1: type, severity updated (rev "),
        "{said}"
    );
    assert_eq!(item(&work, "I-1")["type"], "bug");
    work.worker_ok(&["item", "set", "I-1", "--assignee", "owner"]);
    assert_eq!(item(&work, "I-1")["assignee"], "owner");
    work.worker_ok(&["item", "set", "I-1", "--assignee", "none"]);
    assert_eq!(item(&work, "I-1")["assignee"], Value::Null);
    work.worker_ok(&["item", "add", "Release", "--type", "epic"]);
    work.worker_ok(&["item", "set", "I-1", "--parent", "I-2"]);
    assert_eq!(item(&work, "I-1")["parent"], "I-2");
    work.worker_ok(&["item", "set", "I-1", "--parent", "none"]);
    assert_eq!(item(&work, "I-1")["parent"], Value::Null);
    let wrong = work.worker(&["item", "set", "I-1", "--field", "mood=ok"]);
    assert!(
        wrong.err.contains("mood is not a field here"),
        "{}",
        wrong.err
    );
}

#[test]
fn status_takes_a_lane_or_a_stage_and_refuses_a_word_that_is_neither() {
    let work = Work::new();
    work.worker_ok(&["item", "add", "Release", "--type", "epic"]);
    let said = work.worker_ok(&["item", "status", "I-2", "active"]);
    assert!(said.starts_with("I-2: stage updated (rev "), "{said}");
    let epic = item(&work, "I-2");
    assert_eq!(
        (epic["stage"].as_str(), epic["status"].as_str()),
        (Some("active"), Some("doing"))
    );
    let lane = work.worker_ok(&["item", "status", "I-1", "doing"]);
    assert!(lane.starts_with("I-1: status updated (rev "), "{lane}");
    let word = work.worker(&["item", "status", "I-1", "finished"]);
    assert!(
        word.err
            .contains("status must be one of todo, doing, review, done, dropped."),
        "{}",
        word.err
    );
}

#[test]
fn a_done_rule_that_refuses_needs_force_and_force_leaves_a_decision() {
    let work = Work::new();
    work.worker_ok(&["item", "add", "Release", "--type", "epic"]);
    work.worker_ok(&["item", "add", "Story", "--type", "story", "--parent", "I-2"]);
    let refused = work.worker(&["item", "status", "I-2", "done"]);
    assert_eq!(refused.code, 1);
    assert!(
        refused.err.contains("children not done or dropped: I-3"),
        "{}",
        refused.err
    );
    assert_eq!(item(&work, "I-2")["status"], "todo");
    let no_reason = work.worker(&["item", "status", "I-2", "done", "--force"]);
    assert!(
        no_reason.err.contains("--force needs a reason"),
        "{}",
        no_reason.err
    );
    work.worker_ok(&["item", "status", "I-2", "done", "--force", "scope was cut"]);
    assert_eq!(item(&work, "I-2")["status"], "done");
    let kept = decisions_about(&work, "I-2");
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0]["text"], "Forced I-2 to done: scope was cut");
}

#[test]
fn force_keeps_the_revision_check_of_if_rev() {
    let work = Work::new();
    work.worker_ok(&["item", "add", "Release", "--type", "epic"]);
    work.worker_ok(&["item", "add", "Story", "--type", "story", "--parent", "I-2"]);
    work.worker_ok(&["item", "set", "I-2", "--title", "Release 2"]);
    let ran = work.worker(&[
        "item",
        "status",
        "I-2",
        "done",
        "--force",
        "scope was cut",
        "--if-rev",
        "1",
    ]);
    assert_eq!(ran.code, 1);
    assert!(ran.err.contains("I-2 changed since rev 1"), "{}", ran.err);
    assert_eq!(item(&work, "I-2")["status"], "todo");
}

#[test]
fn a_rule_that_only_warns_prints_the_warning_after_the_line() {
    let work = Work::new();
    work.worker_ok(&["item", "add", "Chore", "--type", "task", "--ac", "Checked"]);
    let said = work.worker_ok(&["item", "status", "I-2", "done"]);
    let lines: Vec<&str> = said.lines().collect();
    assert!(lines[0].starts_with("I-2: status updated (rev "), "{said}");
    assert_eq!(lines[1], "warning: criteria still open: AC1");
    assert_eq!(item(&work, "I-2")["status"], "done");
}

/// An epic I-2 with a story I-3 under it and a task I-4 under the story.
fn small_tree() -> Work {
    let work = Work::new();
    work.worker_ok(&["item", "add", "Release", "--type", "epic"]);
    work.worker_ok(&["item", "add", "Story", "--type", "story", "--parent", "I-2"]);
    work.worker_ok(&[
        "item", "add", "Chore", "--type", "task", "--parent", "I-3", "--ac", "One",
    ]);
    work
}

#[test]
fn tree_prints_one_line_per_item_with_the_children_indented() {
    let work = small_tree();
    let said = work.worker_ok(&["item", "tree", "I-2"]);
    let want = "I-2 [epic] todo  WO  Release  0/1\n  I-3 [story] todo  WO  Story  0/1\n    I-4 [task] todo  WO  Chore  0/1\n";
    assert_eq!(said, want);
    let all = work.worker_ok(&["item", "tree"]);
    assert!(all.starts_with("I-1 [task] todo  WO  Login\n"), "{all}");
    assert!(
        all.contains("\nI-2 [epic] todo  WO  Release  0/1\n"),
        "{all}"
    );
}

#[test]
fn tree_narrows_by_type_assignee_and_open() {
    let work = small_tree();
    work.worker_ok(&["item", "status", "I-4", "done"]);
    work.worker_ok(&["item", "set", "I-3", "--assignee", "owner"]);
    let story = work.worker_ok(&["item", "tree", "--type", "story"]);
    assert_eq!(story, "I-3 [story] todo  ME  Story  1/1\n");
    let owner = work.worker_ok(&["item", "tree", "--assignee", "owner"]);
    assert_eq!(owner, "I-3 [story] todo  ME  Story  1/1\n");
    let open = work.worker_ok(&["item", "tree", "I-2", "--open"]);
    assert!(!open.contains("I-4"), "{open}");
    assert!(open.contains("I-3 [story]"), "{open}");
    let none = work.worker_ok(&["item", "tree", "--type", "debt"]);
    assert_eq!(none, "no items.\n");
    let wrong = work.worker(&["item", "tree", "--type", "saga"]);
    assert!(wrong.err.contains("type must be one of"), "{}", wrong.err);
}

#[test]
fn list_filters_by_type_and_assignee_and_keeps_its_columns() {
    let work = small_tree();
    let all = work.worker_ok(&["item", "list"]);
    assert!(
        all.starts_with("item  status  thread  ac   title\n"),
        "{all}"
    );
    let epics = work.worker_ok(&["item", "list", "--type", "epic"]);
    assert!(epics.contains("I-2") && !epics.contains("I-1"), "{epics}");
    let tasks = work.worker_ok(&["item", "list", "--type", "task"]);
    assert!(tasks.contains("I-1") && tasks.contains("I-4"), "{tasks}");
    work.worker_ok(&["item", "set", "I-3", "--assignee", "owner"]);
    let owned = work.worker_ok(&["item", "list", "--assignee", "owner"]);
    assert!(owned.contains("I-3") && !owned.contains("I-2"), "{owned}");
    let mine = work.worker_ok(&["item", "list", "--assignee", "me"]);
    assert!(mine.contains("I-2") && !mine.contains("I-3"), "{mine}");
}

#[test]
fn types_lists_the_registry_and_describes_one_type() {
    let work = Work::new();
    let all = work.worker_ok(&["types"]);
    assert!(all.starts_with("type"), "{all}");
    for id in [
        "epic",
        "story",
        "task",
        "bug",
        "debt",
        "test-plan",
        "test-case",
    ] {
        assert!(all.contains(id), "{id} missing: {all}");
    }
    assert!(all.contains("Under an item: question Q-n"), "{all}");
    let bug = work.worker_ok(&["types", "bug"]);
    assert!(
        bug.starts_with("bug (Bug), parent: epic, story\nStages:\n"),
        "{bug}"
    );
    assert!(
        bug.contains("  verifying (lane review), then closed | fixing"),
        "{bug}"
    );
    assert!(
        bug.contains("  --severity one of S1|S2|S3|S4, required"),
        "{bug}"
    );
    assert!(bug.contains("Done refuses unless: verified."), "{bug}");
    let none = work.worker(&["types", "saga"]);
    assert_eq!(none.code, 1);
    assert!(
        none.err.contains("no type \"saga\": the types are epic"),
        "{}",
        none.err
    );
}

const PNG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

fn with_picture(work: &Work, name: &str, bytes: &[u8]) {
    std::fs::write(work.cwd.join(name), bytes).expect("picture");
}

fn pictures(work: &Work, id: &str) -> usize {
    item(work, id)["images"].as_array().map_or(0, Vec::len)
}

#[test]
fn add_and_set_keep_a_picture_with_its_caption_and_keep_the_same_bytes_once() {
    let work = Work::new();
    with_picture(&work, "shot.png", &PNG);
    let said = work.worker_ok(&[
        "item",
        "add",
        "With picture",
        "--image",
        "shot.png",
        "The form",
    ]);
    assert_eq!(said, "I-2: With picture (worker, 0 AC)\n");
    assert_eq!(pictures(&work, "I-2"), 1);
    assert_eq!(item(&work, "I-2")["images"][0]["caption"], "The form");
    let file = work.home.coord_home().join("alpha/proofs/I-2-1.png");
    assert!(file.is_file(), "{}", file.display());
    let again = work.worker_ok(&["item", "set", "I-2", "--image", "shot.png", "Again"]);
    assert!(again.starts_with("I-2: images updated (rev "), "{again}");
    assert_eq!(pictures(&work, "I-2"), 1);
    with_picture(&work, "other.png", &[0x89, b'P', b'N', b'G', 1]);
    work.worker_ok(&[
        "item",
        "set",
        "I-2",
        "--title",
        "Renamed",
        "--image",
        "other.png",
        "Two",
    ]);
    assert_eq!(pictures(&work, "I-2"), 2);
}

#[test]
fn a_picture_needs_a_caption_a_file_and_a_known_kind() {
    let work = Work::new();
    with_picture(&work, "shot.png", &PNG);
    with_picture(&work, "notes.txt", b"text");
    let no_caption = work.worker(&["item", "add", "Pic", "--image", "shot.png"]);
    assert_eq!(no_caption.code, 1);
    assert!(
        no_caption.err.contains("each image needs a caption"),
        "{}",
        no_caption.err
    );
    let missing = work.worker(&["item", "add", "Pic", "--image", "gone.png", "Gone"]);
    assert!(missing.err.contains("no image at"), "{}", missing.err);
    let text = work.worker(&["item", "set", "I-1", "--image", "notes.txt", "Notes"]);
    assert_eq!(text.code, 1);
    assert_eq!(pictures(&work, "I-1"), 0);
    assert_eq!(
        work.board()["items"]
            .as_object()
            .map_or(0, serde_json::Map::len),
        1
    );
}
