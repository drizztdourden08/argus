// test-tag: critical

use serde_json::{Value, json};

use super::{Boards, MAX_DEPTH, Node, summaries, walk};
use crate::blocked::CauseKind;

fn board(items: Value) -> Value {
    let mut out = serde_json::Map::new();
    out.insert("items".to_owned(), items);
    Value::Object(out)
}

fn boards(list: &[(&str, Value)]) -> Boards {
    list.iter()
        .map(|(key, board)| ((*key).to_owned(), board.clone()))
        .collect()
}

/// A mirror entry on the requester item.
fn waits(dep: &str, (project, item): (&str, &str), state: &str) -> Value {
    json!({ "dep": dep, "project": project, "item": item, "title": "from the mirror", "state": state, "since": "2026-10-30T08:00:00Z" })
}

/// One request of the ledger: its id, the (project, item) that asks, the one asked, and its state.
struct Row<'a> {
    id: &'a str,
    from: (&'a str, &'a str),
    to: (&'a str, &'a str),
    state: &'a str,
}

fn row<'a>(
    id: &'a str,
    from: (&'a str, &'a str),
    to: (&'a str, &'a str),
    state: &'a str,
) -> Row<'a> {
    Row {
        id,
        from,
        to,
        state,
    }
}

/// A ledger record of a request.
fn ledger(deps: &[Row]) -> Value {
    let mut out = serde_json::Map::new();
    for dep in deps {
        out.insert(
            dep.id.to_owned(),
            json!({
                "id": dep.id, "state": dep.state, "createdAt": "2026-10-29T08:00:00Z", "openedAt": "2026-10-30T09:00:00Z",
                "from": { "project": dep.from.0, "item": dep.from.1 }, "to": { "project": dep.to.0, "item": dep.to.1 },
            }),
        );
    }
    json!({ "deps": out })
}

/// Bxel I-136 waits on Tessera I-70, which waits on Brock I-105.
fn three_boards() -> Boards {
    boards(&[
        (
            "bxel",
            board(json!({"I-136": {
                "title": "Pixel brush", "type": "feature", "status": "doing", "stage": "building",
                "assignee": "lead:bxel", "waitsOn": [waits("D-21", ("tessera", "I-70"), "open")],
            }})),
        ),
        (
            "tessera",
            board(json!({"I-70": {
                "title": "Canvas piece", "type": "feature", "status": "doing", "stage": "building",
                "thread": "ui", "waitsOn": [waits("D-9", ("brock", "I-105"), "open")],
            }})),
        ),
        (
            "brock",
            board(
                json!({"I-105": {"title": "Raster core", "type": "bug", "status": "todo", "stage": "triage"}}),
            ),
        ),
        (
            "_portfolio",
            ledger(&[
                row("D-21", ("bxel", "I-136"), ("tessera", "I-70"), "open"),
                row("D-9", ("tessera", "I-70"), ("brock", "I-105"), "open"),
            ]),
        ),
    ])
}

fn assert_root(root: &Node) {
    assert_eq!(root.project, "bxel");
    assert_eq!(root.title, "Pixel brush");
    assert_eq!(root.kind.as_deref(), Some("feature"));
    assert_eq!(root.lane.as_deref(), Some("doing"));
    assert_eq!(root.stage.as_deref(), Some("building"));
    assert_eq!(root.assignee.as_deref(), Some("lead:bxel"));
    assert_eq!(root.edge, None);
}

fn assert_middle(middle: &Node) {
    assert_eq!(
        (middle.project.as_str(), middle.item.as_str()),
        ("tessera", "I-70")
    );
    assert_eq!(middle.assignee.as_deref(), Some("ui"));
    let edge = middle.edge.clone().expect("edge");
    assert_eq!(edge.kind, CauseKind::DepWaits);
    assert_eq!(edge.id, "D-21");
    assert_eq!(edge.state.as_deref(), Some("open"));
    assert_eq!(edge.since.as_deref(), Some("2026-10-30T09:00:00Z"));
}

#[test]
fn the_chain_across_three_boards_has_depth_three() {
    let trail = walk(&three_boards(), "bxel", "I-136");
    assert_eq!(trail.depth, 3);
    assert_eq!(trail.open, 3);
    assert_root(&trail.root);
    let middle = &trail.root.children[0];
    assert_middle(middle);
    let end = &middle.children[0];
    assert_eq!(
        (end.project.as_str(), end.item.as_str()),
        ("brock", "I-105")
    );
    assert_eq!(end.children.len(), 0);
}

#[test]
fn a_request_the_ledger_does_not_know_is_followed_through_the_mirror() {
    let mut all = three_boards();
    all.remove("_portfolio");
    let trail = walk(&all, "bxel", "I-136");
    assert_eq!(trail.depth, 3);
    let edge = trail.root.children[0].edge.clone().expect("edge");
    assert_eq!(edge.since.as_deref(), Some("2026-10-30T08:00:00Z"));
}

#[test]
fn an_item_that_waits_on_several_things_branches() {
    let all = boards(&[
        (
            "a",
            board(json!({
                "I-1": {"title": "Top", "status": "doing", "blockedBy": ["I-2"], "waitsOn": [
                    waits("D-1", ("b", "I-7"), "open"), waits("D-2", ("c", "I-8"), "open")]},
                "I-2": {"title": "Local", "status": "doing"},
            })),
        ),
        (
            "b",
            board(json!({"I-7": {"title": "On b", "status": "todo"}})),
        ),
        (
            "c",
            board(json!({"I-8": {"title": "On c", "status": "doing"}})),
        ),
    ]);
    let trail = walk(&all, "a", "I-1");
    assert_eq!(trail.root.children.len(), 3);
    assert_eq!(trail.depth, 2);
    assert_eq!(trail.open, 4);
    let kinds: Vec<CauseKind> = trail
        .root
        .children
        .iter()
        .map(|n| n.edge.clone().expect("edge").kind)
        .collect();
    assert_eq!(
        kinds,
        vec![CauseKind::Item, CauseKind::DepWaits, CauseKind::DepWaits]
    );
}

#[test]
fn an_open_question_ends_a_branch_as_a_leaf() {
    let all = boards(&[(
        "a",
        json!({
            "items": {"I-1": {"title": "Top", "status": "doing", "blockedBy": ["I-2"]}, "I-2": {"title": "Mid", "status": "doing"}},
            "entries": {"Q-4": {"item": "I-2", "kind": "question", "status": "open", "text": "Which port?", "createdAt": "2026-10-31T10:00:00Z"}},
        }),
    )]);
    let trail = walk(&all, "a", "I-1");
    assert_eq!(trail.depth, 3);
    assert_eq!(trail.open, 3);
    assert_question_leaf(&trail.root.children[0].children[0]);
}

fn assert_question_leaf(leaf: &Node) {
    assert_eq!(leaf.item, "Q-4");
    assert_eq!(leaf.title, "Which port?");
    assert_eq!(leaf.kind.as_deref(), Some("question"));
    assert_eq!(leaf.lane.as_deref(), Some("open"));
    assert_eq!(leaf.children.len(), 0);
    let edge = leaf.edge.clone().expect("edge");
    assert_eq!((edge.kind, edge.id.as_str()), (CauseKind::Ask, "Q-4"));
    assert_eq!(edge.since.as_deref(), Some("2026-10-31T10:00:00Z"));
}

#[test]
fn a_pending_decision_is_a_leaf_too() {
    let all = boards(&[(
        "a",
        json!({
            "items": {"I-1": {"title": "Top", "status": "doing"}},
            "entries": {"D-5": {"item": "I-1", "kind": "decision", "status": "pending", "text": "Which design?"}},
        }),
    )]);
    let leaf = &walk(&all, "a", "I-1").root.children[0];
    assert_eq!(leaf.kind.as_deref(), Some("decision"));
    assert_eq!(leaf.edge.clone().expect("edge").kind, CauseKind::Decision);
}

#[test]
fn a_cycle_is_shown_once_and_not_followed() {
    let all = boards(&[
        (
            "a",
            board(
                json!({"I-1": {"title": "One", "status": "doing", "waitsOn": [waits("D-1", ("b", "I-2"), "open")]}}),
            ),
        ),
        (
            "b",
            board(
                json!({"I-2": {"title": "Two", "status": "doing", "waitsOn": [waits("D-2", ("a", "I-1"), "open")]}}),
            ),
        ),
    ]);
    let trail = walk(&all, "a", "I-1");
    assert_eq!(trail.depth, 3);
    assert_eq!(trail.open, 2);
    let again = &trail.root.children[0].children[0];
    assert_eq!((again.project.as_str(), again.item.as_str()), ("a", "I-1"));
    assert!(again.cycle);
    assert_eq!(again.children.len(), 0);
    assert!(!trail.root.cycle);
}

#[test]
fn an_item_that_blocks_itself_is_no_cycle_and_no_child() {
    let all = boards(&[(
        "a",
        board(json!({"I-1": {"title": "One", "status": "doing", "blockedBy": ["I-1"]}})),
    )]);
    let trail = walk(&all, "a", "I-1");
    assert_eq!(trail.root.children.len(), 0);
    assert_eq!(trail.depth, 1);
}

#[test]
fn settled_hops_are_skipped() {
    let all = boards(&[
        (
            "a",
            board(json!({
                "I-1": {"title": "Top", "status": "doing", "blockedBy": ["I-2", "I-3"], "waitsOn": [waits("D-1", ("b", "I-7"), "open")]},
                "I-2": {"title": "Open", "status": "doing", "blockedBy": ["I-4"]},
                "I-3": {"title": "Finished", "status": "done", "blockedBy": ["I-4"]},
                "I-4": {"title": "Behind a finished one", "status": "dropped"},
            })),
        ),
        (
            "b",
            board(json!({"I-7": {"title": "Done on b", "status": "done"}})),
        ),
        (
            "_portfolio",
            ledger(&[row("D-1", ("a", "I-1"), ("b", "I-7"), "done")]),
        ),
    ]);
    let trail = walk(&all, "a", "I-1");
    assert_eq!(trail.root.children.len(), 1);
    assert_eq!(trail.root.children[0].item, "I-2");
    assert_eq!(trail.root.children[0].children.len(), 0);
    assert_eq!(trail.depth, 2);
    assert_eq!(trail.open, 2);
}

#[test]
fn a_request_of_another_board_that_waits_on_this_item_is_followed_to_the_requester() {
    let asked =
        json!({"dep": "D-3", "project": "b", "item": "I-9", "title": "Asked", "state": "blocked"});
    let all = boards(&[
        (
            "a",
            board(json!({"I-1": {"title": "Answer", "status": "doing", "requestedBy": [asked]}})),
        ),
        (
            "b",
            board(json!({"I-9": {"title": "Asker", "status": "doing"}})),
        ),
        (
            "_portfolio",
            ledger(&[row("D-3", ("b", "I-9"), ("a", "I-1"), "blocked")]),
        ),
    ]);
    let child = &walk(&all, "a", "I-1").root.children[0];
    assert_eq!((child.project.as_str(), child.item.as_str()), ("b", "I-9"));
    assert_eq!(
        child.edge.clone().expect("edge").kind,
        CauseKind::DepBlocked
    );
}

#[test]
fn a_request_without_an_item_yet_is_a_leaf() {
    let opening = json!({"dep": "D-4", "project": "b", "item": null, "title": "Not started", "state": "opening"});
    let all = boards(&[(
        "a",
        board(json!({"I-1": {"title": "Top", "status": "doing", "waitsOn": [opening]}})),
    )]);
    let leaf = &walk(&all, "a", "I-1").root.children[0];
    assert_eq!(leaf.kind.as_deref(), Some("request"));
    assert_eq!(leaf.title, "Not started");
}

#[test]
fn a_chain_is_followed_no_further_than_the_depth_limit() {
    let items: serde_json::Map<String, Value> = (1..=30)
        .map(|n| {
            let next = format!("I-{}", n + 1);
            (
                format!("I-{n}"),
                json!({"title": "Link", "status": "doing", "blockedBy": [next]}),
            )
        })
        .collect();
    let all = boards(&[("a", board(Value::Object(items)))]);
    let trail = walk(&all, "a", "I-1");
    assert_eq!(trail.depth, MAX_DEPTH + 1);
}

#[test]
fn open_counts_a_shared_node_once_and_leaves_out_nothing_that_is_not_done() {
    let all = boards(&[(
        "a",
        board(json!({
            "I-1": {"title": "Top", "status": "doing", "blockedBy": ["I-2", "I-3"]},
            "I-2": {"title": "Left", "status": "doing", "blockedBy": ["I-4"]},
            "I-3": {"title": "Right", "status": "doing", "blockedBy": ["I-4"]},
            "I-4": {"title": "Shared", "status": "todo"},
        })),
    )]);
    let trail = walk(&all, "a", "I-1");
    assert_eq!(trail.depth, 3);
    assert_eq!(trail.open, 4);
}

#[test]
fn a_start_that_is_done_is_not_open() {
    let all = boards(&[(
        "a",
        board(json!({"I-1": {"title": "Over", "status": "done"}})),
    )]);
    let trail = walk(&all, "a", "I-1");
    assert_eq!(trail.open, 0);
    assert_eq!(trail.depth, 1);
}

#[test]
fn a_missing_board_or_item_is_a_bare_node() {
    let trail = walk(&Boards::new(), "nope", "I-1");
    assert_eq!(trail.depth, 1);
    assert_eq!(trail.root.title, "");
}

#[test]
fn the_summaries_list_the_blocked_unsettled_items_of_a_board() {
    let all = boards(&[(
        "a",
        board(json!({
            "I-1": {"title": "Free", "status": "doing"},
            "I-2": {"title": "Blocked", "status": "doing", "blockedBy": ["I-3"]},
            "I-3": {"title": "Cause", "status": "todo"},
            "I-4": {"title": "Done but linked", "status": "done", "blockedBy": ["I-3"]},
        })),
    )]);
    let found = summaries(&all, "a");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].item, "I-2");
    assert_eq!((found[0].depth, found[0].open), (2, 2));
    assert_eq!(summaries(&all, "missing").len(), 0);
}

#[test]
fn a_trail_serializes_with_camel_names_and_the_type_key() {
    let value = serde_json::to_value(walk(&three_boards(), "bxel", "I-136")).expect("json");
    assert_eq!(value["depth"], json!(3));
    assert_eq!(value["root"]["type"], json!("feature"));
    assert_eq!(
        value["root"]["children"][0]["edge"]["kind"],
        json!("dep-waits")
    );
}
