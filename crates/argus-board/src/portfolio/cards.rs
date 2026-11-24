//! The project cards of the Argus page, computed from each project's board.

use std::collections::HashSet;
use std::path::Path;

use argus_core::config::hex;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::Board;
use crate::agents::live_agent_ids;
use crate::json::{display, nullish, section, truthy};
use crate::launchers::{MASTER, has_master, launcher_def_name};
use crate::names::display_name;
use crate::ops::is_image_icon;
use crate::portfolio::owner_items::{owner_items, waits_on_owner};
use crate::served::master_name;
use crate::store::read_board;

const RECENT: usize = 6;
const ATTENTION: usize = 5;

/// The project's master launcher: what the card's Play starts, by its run name. `None` when the
/// board has none; `play` is null while it has no Play (retired, several lines).
fn master_of(board: &Board) -> Value {
    let name = master_name(board).unwrap_or_else(|| MASTER.to_owned());
    let l = &board["launchers"][name.as_str()];
    if l.is_null() {
        return Value::Null;
    }
    let def = &board["runDefs"][launcher_def_name(&name).as_str()];
    let play = if !truthy(l, "retiredAt") && def["launcher"] == name.as_str() {
        def["name"].clone()
    } else {
        Value::Null
    };
    json!({
        "name": name, "title": l["title"], "cmd": l["cmd"], "current": l["current"] == true,
        "window": l["window"] == true, "play": play, "updatedAt": l["updatedAt"], "by": l["by"],
    })
}

/// What the card shows of a project's records: those of threads that are not paused, and not
/// archived or in the trash.
struct View<'a> {
    live: Vec<&'a Value>,
    paused: usize,
    entries: Vec<&'a Value>,
    actions: Vec<&'a Value>,
    tests: Vec<&'a Value>,
    items: Vec<&'a Value>,
}

fn shown_doc(paused: &HashSet<String>, x: &Value) -> bool {
    !x["thread"].as_str().is_some_and(|t| paused.contains(t))
        && !truthy(x, "archivedAt")
        && !truthy(x, "deletedAt")
}

fn view_of(board: &Board) -> (View<'_>, HashSet<String>) {
    let threads: Vec<&Value> = section(board, "threads").values().collect();
    let paused: HashSet<String> = threads
        .iter()
        .filter(|t| t["status"] == "paused")
        .map(|t| display(&t["id"]))
        .collect();
    let live = threads
        .iter()
        .filter(|t| !matches!(t["status"].as_str(), Some("done" | "archived")))
        .filter(|t| !paused.contains(&display(&t["id"])))
        .copied()
        .collect();
    let visible = |coll: &str| -> Vec<&Value> {
        section(board, coll)
            .values()
            .filter(|x| shown_doc(&paused, x))
            .collect()
    };
    let view = View {
        live,
        paused: paused.len(),
        entries: visible("entries"),
        actions: visible("actions"),
        tests: visible("tests"),
        items: visible("items"),
    };
    (view, paused)
}

fn is_blocked(t: &Value) -> bool {
    t["status"] == "blocked" || truthy(&t["flags"], "blocked")
}

/// The shown items in stage done (else status done) that belong to a live thread.
fn done_of_live_threads(v: &View) -> usize {
    let live: HashSet<String> = v.live.iter().map(|t| display(&t["id"])).collect();
    v.items
        .iter()
        .filter(|i| i["stage"] == "done" || (!truthy(i, "stage") && i["status"] == "done"))
        .filter(|i| i["thread"].as_str().is_some_and(|t| live.contains(t)))
        .count()
}

fn counts_of(board: &Board, v: &View, open_questions: usize) -> Value {
    let live = &v.live;
    let pending_actions = v
        .actions
        .iter()
        .filter(|a| a["status"] == "pending" && waits_on_owner(a))
        .count();
    let manual = v
        .tests
        .iter()
        .filter(|t| t["mode"] == "manual" && t["status"] == "pending")
        .count();
    let items = |status: &str| v.items.iter().filter(|i| i["status"] == status).count();
    let tests = |statuses: &[&str]| {
        v.tests
            .iter()
            .filter(|t| t["status"].as_str().is_some_and(|s| statuses.contains(&s)))
            .count()
    };
    json!({
        "threads": live.iter().filter(|t| t["role"] != "lead").count(),
        "working": live.iter().filter(|t| t["activity"] == "working").count(),
        "blocked": live.iter().filter(|t| is_blocked(t)).count(),
        "needsYou": open_questions + pending_actions + manual,
        "itemsOpen": v.items.iter().filter(|i| !matches!(i["status"].as_str(), Some("done" | "dropped"))).count(),
        "doing": items("doing"), "review": items("review"), "done": done_of_live_threads(v),
        "testsPass": tests(&["pass"]), "tests": v.tests.len(),
        "testsFailing": tests(&["fail", "partial", "blocked"]),
        "agents": live.iter().map(|t| live_agent_ids(board, t["id"].as_str()).len()).sum::<usize>(),
        "paused": v.paused,
    })
}

fn attention_of(v: &View, open_questions: &[&Value]) -> Vec<String> {
    let blocked = v.live.iter().filter(|t| is_blocked(t)).map(|t| {
        let who = nullish(t, "session")
            .or_else(|| nullish(t, "id"))
            .unwrap_or_default();
        let note = if truthy(t, "note") {
            format!(": {}", display(&t["note"]))
        } else {
            String::new()
        };
        format!("{who} is blocked{note}")
    });
    let decisions = open_questions
        .iter()
        .filter(|e| e["needs"] == "decision")
        .map(|e| {
            format!(
                "{} needs a decision: {}",
                display(&e["id"]),
                display(&e["text"])
            )
        });
    let failing = v
        .tests
        .iter()
        .filter(|t| t["status"] == "fail")
        .map(|t| format!("{} failing: {}", display(&t["id"]), display(&t["title"])));
    blocked
        .chain(decisions)
        .chain(failing)
        .take(ATTENTION)
        .collect()
}

fn recent_of(board: &Board, paused: &HashSet<String>) -> Vec<Value> {
    let mut lines: Vec<(String, Value)> = Vec::new();
    for log in section(board, "log").values() {
        for line in log["entries"].as_array().into_iter().flatten() {
            if shown_doc(paused, line) {
                let ts = display(&line["ts"]);
                let mut with_subject = line.clone();
                crate::json::set(&mut with_subject, "subject", log["subject"].clone());
                lines.push((ts, with_subject));
            }
        }
    }
    lines.sort_by(|a, b| b.0.cmp(&a.0));
    lines
        .into_iter()
        .take(RECENT)
        .map(|(_, l)| json!({"ts": l["ts"], "thread": l["thread"], "subject": l["subject"], "text": l["text"]}))
        .collect()
}

fn lead_of(coord: Option<&Value>) -> Value {
    coord.map_or(Value::Null, |c| {
        json!({
            "session": c["session"], "link": c["link"], "lastSeen": c["lastSeen"],
            "activity": c["activity"], "note": c["note"],
        })
    })
}

/// The icon a card shows: a brand or a curated name as is, an image as the url that serves it. The
/// url carries a short hash of the image so a new image is fetched anew.
fn icon_of(key: &str, board: &Board) -> Option<String> {
    let icon = nullish(board, "icon")?;
    if !is_image_icon(&icon) {
        return Some(icon);
    }
    let digest = hex(&Sha256::digest(icon.as_bytes()));
    Some(format!("/api/p/{key}/icon?v={}", &digest[..8]))
}

fn card_of(home: &Path, key: &str, reg: &Value, board: &Board) -> Value {
    let (view, paused) = view_of(board);
    let coord = view.live.iter().find(|t| t["role"] == "lead").copied();
    let open: Vec<&Value> = view
        .entries
        .iter()
        .filter(|e| e["kind"] == "question" && e["status"] == "open")
        .copied()
        .collect();
    let root = if reg["root"].is_null() {
        coord.map_or(Value::Null, |c| c["root"].clone())
    } else {
        reg["root"].clone()
    };
    let mut card = Map::new();
    let mut put = |k: &str, v: Value| card.insert(k.to_owned(), v);
    put("key", json!(key));
    put(
        "name",
        json!(display_name(
            home,
            key,
            board["name"].as_str(),
            reg["name"].as_str()
        )),
    );
    put(
        "state",
        json!(if coord.is_some() {
            "running"
        } else {
            "no lead"
        }),
    );
    put("root", root);
    if let Some(icon) = icon_of(key, board) {
        put("icon", json!(icon));
    }
    put("url", json!(format!("/p/{key}/")));
    put("lead", lead_of(coord));
    put("counts", counts_of(board, &view, open.len()));
    put("attention", json!(attention_of(&view, &open)));
    put("ownerItems", json!(owner_items(&view.items, &view.actions)));
    put("recent", Value::Array(recent_of(board, &paused)));
    put("master", master_of(board));
    if !has_master(board) {
        put("masterMissing", json!(true));
    }
    if let Some(at) = board.get("updatedAt") {
        put("updatedAt", at.clone());
    }
    Value::Object(card)
}

/// The card of a project for the Argus page. `board` is the project's board when the caller
/// holds it; `None` reads it from the coord home, and a project without a board is a "not
/// started" card.
#[must_use]
pub fn summarize(home: &Path, key: &str, p: &Board, board: Option<&Board>) -> Value {
    let loaded = if board.is_none() {
        read_board(home, key).ok().flatten()
    } else {
        None
    };
    let board = board.or(loaded.as_ref());
    let reg = match &p["projects"][key] {
        Value::Null => json!({"key": key, "name": key}),
        r => r.clone(),
    };
    let Some(board) = board else {
        return json!({
            "key": key, "name": display_name(home, key, None, reg["name"].as_str()),
            "state": "not started", "root": reg["root"], "url": format!("/p/{key}/"),
        });
    };
    card_of(home, key, &reg, board)
}
