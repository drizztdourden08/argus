//! `coord worktree rule add | rule rm | rules`: tell Argus where the project's worktrees live.
//! The rules are on the board; the dashboard looks at those places and lists what it finds.

use argus_board::json::array_mut;
use argus_core::js;
use serde_json::{Value, json};

use super::opts::{list_opt, text_opt};
use super::path::has_wildcard;
use super::rule_add::{RuleInput, add_rule, audit};
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::out;
use crate::cli::route::Call;
use crate::verbs::remote::support::link::project_of;
use crate::verbs::remote::support::scope::{locked, require_board};
use crate::verbs::remote::support::text::{cell, num, say_all};

const DEFAULT_STALE_DAYS: f64 = 3.0;

fn stale_of(rule: &Value) -> String {
    match rule.get("staleDays") {
        None | Some(Value::Null) => num(DEFAULT_STALE_DAYS),
        Some(days) => cell(Some(days)),
    }
}

fn joined(rule: &Value, field: &str) -> String {
    let words: Vec<String> = rule[field]
        .as_array()
        .map(|a| a.iter().map(|v| cell(Some(v))).collect())
        .unwrap_or_default();
    words.join(" ")
}

fn filters_text(rule: &Value) -> String {
    let part = |field: &str| {
        let words = joined(rule, field);
        if words.is_empty() {
            String::new()
        } else {
            format!(", {field} {words}")
        }
    };
    format!("{}{}", part("include"), part("exclude"))
}

fn describe(rule: &Value) -> String {
    let note = if js::truthy(&rule["note"]) {
        format!(", {}", cell(rule.get("note")))
    } else {
        String::new()
    };
    format!(
        "{} {}{}, tells {}, stale after {} days{note}",
        cell(rule.get("kind")),
        cell(rule.get("path")),
        filters_text(rule),
        cell(rule.get("thread")),
        stale_of(rule)
    )
}

fn input_of(call: &Call<'_>) -> Result<RuleInput, CoordError> {
    let Some(path) = call.positional.first().cloned() else {
        return Err(CoordError::refuse(
            "give the PARENT FOLDER of the worktrees: coord worktree rule add <path> [--kind folder|repo] [--include \"<name-glob>\"] [--exclude \"<name-glob>\"] [--thread t] [--stale-days n] [--note \"...\"].",
        ));
    };
    let o = &call.options;
    Ok(RuleInput {
        path,
        kind: text_opt(o, "kind")?,
        include: list_opt(o, "include")?,
        exclude: list_opt(o, "exclude")?,
        thread: text_opt(o, "thread")?,
        stale_days: text_opt(o, "stale-days")?,
        note: text_opt(o, "note")?,
    })
}

/// `coord worktree rule add <path> ...`.
pub(super) fn rule_add(call: &mut Call<'_>) -> Outcome {
    let input = input_of(call)?;
    let project = project_of(call).to_owned();
    locked(call, &project, false, |board, scope| {
        let who = scope.begin(board)?;
        let added = add_rule(board, &who, &input)?;
        let id = cell(added.rule.get("id"));
        let rule = describe(&added.rule);
        let line = if added.created {
            format!("{id}: rule added ({rule}).")
        } else if added.changed.is_empty() {
            format!("{id}: unchanged ({rule}).")
        } else {
            format!("{id}: rule updated ({}): {rule}.", added.changed.join(", "))
        };
        scope.say(&[&line]);
        if added.created {
            scope.say(&["Argus scans it within a minute (then whenever a worktree changes, and every 30 minutes in any case); coord worktree list shows what it found, and you get a worktree-stale notice when one needs a decision."]);
        }
        let path = cell(added.rule.get("path"));
        if !has_wildcard(&path) && !std::path::Path::new(&path).exists() {
            scope.say(&[&format!("warning: {path} does not exist on this machine right now; the rule is kept and the page shows it as missing.")]);
        }
        Ok(())
    })
}

/// `coord worktree rule rm <W-n>`.
pub(super) fn rule_rm(call: &mut Call<'_>) -> Outcome {
    let Some(id) = call.positional.first().filter(|i| !i.is_empty()).cloned() else {
        return Err(CoordError::refuse(
            "name the rule: coord worktree rule rm <W-n>.",
        ));
    };
    let project = project_of(call).to_owned();
    locked(call, &project, false, |board, scope| {
        let who = scope.begin(board)?;
        let rules = array_mut(board, "worktreeRules");
        let Some(at) = rules.iter().position(|r| r["id"] == id.as_str()) else {
            return Err(CoordError::refuse(format!(
                "no worktree rule \"{id}\" (coord worktree rules lists them)."
            )));
        };
        let gone = rules.remove(at);
        let path = cell(gone.get("path"));
        audit(board, &who, &format!("removed worktree rule {id}: {path}"));
        scope.say(&[&format!(
            "{id}: rule removed ({path}). Its worktrees leave the Worktrees tab at the next scan."
        )]);
        Ok(())
    })
}

/// A rule as the listing shows it: the lists always there, and why it is invalid.
fn shown(rule: &Value) -> Value {
    let mut shown = rule.clone();
    for field in ["include", "exclude"] {
        if !shown[field].is_array() {
            shown[field] = json!([]);
        }
    }
    let invalid = has_wildcard(&cell(rule.get("path")));
    shown["invalid"] = if invalid {
        json!("invalid: wildcard path")
    } else {
        Value::Null
    };
    shown
}

fn row(rule: &Value, now: i64) -> Vec<String> {
    let id = cell(rule.get("id"));
    let last = match rule["invalid"].as_str() {
        Some(why) => format!(
            "{why} (ignored: a rule path is the parent folder, no wildcards; coord worktree rule rm {id}, then add the folder with --include)"
        ),
        None => cell(rule.get("note")),
    };
    vec![
        id,
        cell(rule.get("kind")),
        cell(rule.get("path")),
        joined(rule, "include"),
        joined(rule, "exclude"),
        cell(rule.get("thread")),
        format!("{}d", stale_of(rule)),
        format!(
            "{} {}",
            cell(rule.get("addedBy")),
            out::ago(rule["addedAt"].as_str(), now)
        ),
        last,
    ]
}

/// `coord worktree rules [--json]`.
pub(super) fn rules(call: &mut Call<'_>) -> Outcome {
    let board = require_board(call, project_of(call))?;
    let list: Vec<Value> = board["worktreeRules"]
        .as_array()
        .map(|a| a.iter().map(shown).collect())
        .unwrap_or_default();
    if call.options.flag("json") {
        out::as_json(&mut call.inv.stdout, &json!({ "rules": list }));
    } else if list.is_empty() {
        call.inv.say(&["no worktree rules yet: coord worktree rule add <parent folder> [--kind folder|repo] [--include \"<name-glob>\"] (the Worktrees tab stays empty until you tell Argus where the worktrees live)."]);
    } else {
        let now = call.inv.clock.now_ms();
        let mut rows = vec![
            [
                "id", "kind", "path", "include", "exclude", "thread", "stale", "set", "note",
            ]
            .map(str::to_owned)
            .to_vec(),
        ];
        rows.extend(list.iter().map(|r| row(r, now)));
        say_all(call.inv, &[out::table(&rows)]);
    }
    Ok(())
}
