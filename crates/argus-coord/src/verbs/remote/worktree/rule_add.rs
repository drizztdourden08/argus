//! `addRule` of `worktrees.mjs`: a worktree rule says where a project's worktrees live. The same
//! path, kind and name lists again update that rule; other lists make another rule.

use argus_board::Board;
use argus_board::json::{array_mut, map_mut, strict_eq};
use argus_board::limits::require_text_max;
use argus_board::store::{LogLine, append_log};
use argus_board::text::clip_text;
use argus_core::clock::now_iso;
use argus_core::js;
use serde_json::{Value, json};

use super::path::{has_wildcard, path_key, validate_path};
use crate::cli::exit::CoordError;
use crate::who::Caller;

const MAX_RULES: usize = 40;
const MAX_FILTERS: usize = 8;
const MAX_FILTER: usize = 100;
const NOTE_MAX: usize = 300;
const WILDCARDS: &str =
    "a rule path is the parent folder (no wildcards): use --include/--exclude to choose names.";

/// What `rule add` was asked for, as typed.
pub(super) struct RuleInput {
    pub path: String,
    pub kind: Option<String>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub thread: Option<String>,
    pub stale_days: Option<String>,
    pub note: Option<String>,
}

/// What `rule add` did.
pub(super) struct Added {
    pub rule: Value,
    pub created: bool,
    pub changed: Vec<&'static str>,
}

/// An audit line in the board's log.
pub(super) fn audit(board: &mut Board, who: &Caller, text: &str) {
    let line = LogLine {
        thread: &who.actor,
        item: None,
        text: &clip_text(text, 120),
    };
    append_log(board, "audit", &line);
}

fn filters_of(flag: &str, value: Option<&[String]>) -> Result<Vec<String>, CoordError> {
    let list: Vec<&str> = value
        .unwrap_or_default()
        .iter()
        .map(|g| js::trim(g))
        .collect();
    if list.len() > MAX_FILTERS {
        return Err(CoordError::refuse(format!(
            "--{flag} may be given {MAX_FILTERS} times at most."
        )));
    }
    let mut kept: Vec<String> = Vec::new();
    for g in list {
        check_filter(flag, g)?;
        if !kept.iter().any(|k| k == g) {
            kept.push(g.to_owned());
        }
    }
    Ok(kept)
}

fn check_filter(flag: &str, g: &str) -> Result<(), CoordError> {
    let len = js::len(g);
    if g.is_empty() {
        return Err(CoordError::refuse(format!(
            "--{flag} takes a name pattern such as \"web-*\"."
        )));
    }
    if len > MAX_FILTER {
        return Err(CoordError::refuse(format!(
            "--{flag} \"{}...\" is {len} characters, the limit is {MAX_FILTER}.",
            js::slice(g, 20)
        )));
    }
    if g.contains(['/', '\\']) {
        return Err(CoordError::refuse(format!(
            "--{flag} \"{g}\" holds a slash: it matches ONE folder name directly inside the rule's folder, not a path."
        )));
    }
    if g.chars().any(|c| c <= '\u{1f}' || c == '\u{7f}') {
        return Err(CoordError::refuse(format!(
            "--{flag} holds a control character."
        )));
    }
    Ok(())
}

/// The id of a live thread of the board, else the refusal.
fn live_thread(board: &Board, id: &str) -> Result<String, CoordError> {
    let thread = &board["threads"][id];
    if thread.is_null() || js::truthy(&thread["deletedAt"]) {
        return Err(CoordError::refuse(format!(
            "no thread \"{id}\" on this board to tell about stale worktrees."
        )));
    }
    Ok(match thread["id"].as_str() {
        Some(own) => own.to_owned(),
        None => id.to_owned(),
    })
}

fn stale_days_of(text: &str) -> Result<Value, CoordError> {
    if text.is_empty() {
        return Ok(Value::Null);
    }
    let n = crate::verbs::remote::support::text::number_of(text);
    crate::verbs::remote::support::text::int_within(n, 1.0, 365.0)
        .map(|days| json!(days))
        .ok_or_else(|| {
            CoordError::refuse("--stale-days takes a whole number of days from 1 to 365.")
        })
}

fn note_of(text: &str) -> Result<Value, CoordError> {
    if js::trim(text).is_empty() {
        return Ok(Value::Null);
    }
    Ok(json!(require_text_max("note", &json!(text), NOTE_MAX)?))
}

fn same_list(stored: &Value, list: &[String]) -> bool {
    let mut have: Vec<String> = stored
        .as_array()
        .map(|a| {
            a.iter()
                .map(|v| v.as_str().unwrap_or_default().to_owned())
                .collect()
        })
        .unwrap_or_default();
    let mut want = list.to_vec();
    have.sort();
    want.sort();
    have == want
}

/// The rule that has this kind, path and name lists already.
fn find_same(
    rules: &[Value],
    kind: &str,
    path: &str,
    lists: (&[String], &[String]),
) -> Option<usize> {
    rules.iter().position(|r| {
        r["kind"] == kind
            && path_key(r["path"].as_str().unwrap_or_default()) == path_key(path)
            && same_list(&r["include"], lists.0)
            && same_list(&r["exclude"], lists.1)
    })
}

/// The fields a rule that exists is changed by.
struct Patch {
    thread: Option<Value>,
    stale_days: Option<Value>,
    note: Option<Value>,
}

fn update(have: &mut Value, patch: Patch) -> Vec<&'static str> {
    let mut changed = Vec::new();
    for (field, value) in [
        ("thread", patch.thread),
        ("staleDays", patch.stale_days),
        ("note", patch.note),
    ] {
        let Some(value) = value else {
            continue;
        };
        if have.get(field).is_some_and(|now| strict_eq(now, &value)) {
            continue;
        }
        have[field] = value;
        changed.push(field);
    }
    changed
}

/// Adds a rule, or changes the one with the same kind, path and name lists.
///
/// # Errors
/// Any refusal of the arguments, the 40-rule cap, or a thread that is not on the board.
pub(super) fn add_rule(
    board: &mut Board,
    who: &Caller,
    input: &RuleInput,
) -> Result<Added, CoordError> {
    if has_wildcard(&input.path) {
        return Err(CoordError::refuse(WILDCARDS));
    }
    let path = validate_path(&input.path)?;
    let kind = input.kind.as_deref().map_or("folder", js::trim);
    if kind != "folder" && kind != "repo" {
        return Err(CoordError::refuse("--kind must be one of folder, repo."));
    }
    let include = filters_of("include", input.include.as_deref())?;
    let exclude = filters_of("exclude", input.exclude.as_deref())?;
    if kind == "repo" && !(include.is_empty() && exclude.is_empty()) {
        return Err(CoordError::refuse(
            "--include / --exclude choose folder names inside a folder rule; a repo rule lists the worktrees git knows.",
        ));
    }
    let thread = input
        .thread
        .as_deref()
        .map(|t| live_thread(board, t))
        .transpose()?;
    let stale_days = input.stale_days.as_deref().map(stale_days_of).transpose()?;
    let note = input.note.as_deref().map(note_of).transpose()?;
    let rules = array_mut(board, "worktreeRules");
    if let Some(at) = find_same(rules, kind, &path, (&include, &exclude)) {
        let delta = Patch {
            thread: thread.map(|t| json!(t)),
            stale_days,
            note,
        };
        let changed = update(&mut rules[at], delta);
        let rule = rules[at].clone();
        if !changed.is_empty() {
            let text = format!(
                "updated worktree rule {} ({})",
                rule["id"].as_str().unwrap_or_default(),
                changed.join(", ")
            );
            audit(board, who, &text);
        }
        return Ok(Added {
            rule,
            created: false,
            changed,
        });
    }
    if rules.len() >= MAX_RULES {
        return Err(CoordError::refuse(format!(
            "a project may have {MAX_RULES} worktree rules at most: remove one first (coord worktree rule rm <id>)."
        )));
    }
    create(
        board,
        who,
        (kind, path),
        (include, exclude),
        (thread, stale_days, note),
    )
}

fn create(
    board: &mut Board,
    who: &Caller,
    (kind, path): (&str, String),
    (include, exclude): (Vec<String>, Vec<String>),
    (thread, stale_days, note): (Option<String>, Option<Value>, Option<Value>),
) -> Result<Added, CoordError> {
    let n = board["counters"]["worktreeRule"].as_i64().unwrap_or(0) + 1;
    map_mut(board, "counters").insert("worktreeRule".to_owned(), json!(n));
    let thread = match thread {
        Some(t) => t,
        None => live_thread(board, &who.thread)?,
    };
    let rule = json!({
        "id": format!("W-{n}"), "path": path, "kind": kind, "include": include,
        "exclude": exclude, "thread": thread, "note": note.unwrap_or(Value::Null),
        "staleDays": stale_days.unwrap_or(Value::Null), "addedBy": who.actor, "addedAt": now_iso(),
    });
    array_mut(board, "worktreeRules").push(rule.clone());
    let lists = |label: &str, list: &[String]| {
        if list.is_empty() {
            String::new()
        } else {
            format!(" {label} {}", list.join(","))
        }
    };
    let text = format!(
        "added worktree rule W-{n}: {kind} {path}{}{}",
        lists("include", &include),
        lists("exclude", &exclude)
    );
    audit(board, who, &text);
    Ok(Added {
        rule,
        created: true,
        changed: Vec::new(),
    })
}
