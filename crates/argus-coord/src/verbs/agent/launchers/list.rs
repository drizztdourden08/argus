//! `coord launcher list [--all] [--json]`: the live launchers, master first, then the current ones
//! (`--all` adds the retired).

use std::cmp::Ordering;

use argus_board::Board;
use argus_board::json::{display, section, shown, truthy};
use argus_board::served::is_master;
use argus_launchers::launcher::{EVERY_LEGACY, MASTER, duration_text, has_master};
use serde_json::{Value, json};

use super::row::{Row, row_of};
use crate::cli::Call;
use crate::cli::exit::Outcome;
use crate::cli::out::{ago, as_json, table};
use crate::verbs::agent::base::read;
use crate::verbs::agent::base::show::{cell, clip};
use crate::verbs::agent::rules::typed_board;
use argus_core::collate::compare;

const HEAD: [&str; 9] = [
    "name",
    "kind",
    "current",
    "for",
    "branch",
    "set",
    "used every",
    "command line",
    "",
];

/// Master first, then the current ones, then the rest by name.
fn by_pin(board: &Board, a: &Value, b: &Value) -> Ordering {
    let pin = |l: &Value| (is_master(board, &display(&l["name"])), truthy(l, "current"));
    let (pa, pb) = (pin(a), pin(b));
    pb.0.cmp(&pa.0)
        .then_with(|| pb.1.cmp(&pa.1))
        .then_with(|| compare(&display(&a["name"]), &display(&b["name"]), true))
}

/// The usage column: `every 7d, unused 2d STALE`, `for bundle b4`, or nothing for master.
fn usage_cell(row: &Row, now_ms: i64) -> String {
    if truthy(&row.rec, "retiredAt") {
        return String::new();
    }
    match (&row.usage, &row.joined) {
        (Some(u), _) if row.standalone => {
            let every = if u.legacy {
                format!("{} (default)", u.use_every)
            } else {
                u.use_every.clone()
            };
            let stale = if u.stale { " STALE" } else { "" };
            format!(
                "{every}, unused {}{stale}",
                duration_text(now_ms - u.last_used_ms)
            )
        }
        (_, Some(bundle)) => format!("for bundle {bundle}"),
        _ => String::new(),
    }
}

/// `\s` of a JavaScript regular expression.
fn is_space(c: char) -> bool {
    (c.is_whitespace() && c != '\u{85}') || c == '\u{feff}'
}

/// `text.replace(/\s+/g, ' ')`: each run of white space becomes one space.
fn collapse(text: &str) -> String {
    let mut out = String::new();
    let mut gap = false;
    for c in text.chars() {
        if is_space(c) {
            if !gap {
                out.push(' ');
            }
            gap = true;
        } else {
            out.push(c);
            gap = false;
        }
    }
    out
}

fn line(row: &Row, now_ms: i64) -> Vec<String> {
    let rec = &row.rec;
    let flag = if truthy(rec, "retiredAt") {
        "retired"
    } else if row.play.is_some() {
        ""
    } else {
        "no Play"
    };
    let cmd = collapse(&cell(&rec["cmd"]));
    vec![
        cell(&rec["name"]),
        cell(&rec["kind"]),
        if truthy(rec, "current") {
            "current"
        } else {
            ""
        }
        .to_owned(),
        cell(&rec["for"]),
        cell(&rec["branch"]),
        format!(
            "{} {}",
            shown(rec.get("by")),
            ago(rec["updatedAt"].as_str(), now_ms)
        ),
        usage_cell(row, now_ms),
        clip(&cmd, 60),
        flag.to_owned(),
    ]
}

/// `coord launcher list`.
///
/// # Errors
/// A launcher or bundle record that cannot be read.
pub fn run(call: &mut Call<'_>) -> Outcome {
    read(call, |board, run| {
        let all = run.options.flag("all");
        let typed = typed_board(board)?;
        let mut recs: Vec<&Value> = section(board, "launchers")
            .values()
            .filter(|l| all || !truthy(l, "retiredAt"))
            .collect();
        recs.sort_by(|a, b| by_pin(board, a, b));
        let rows: Vec<Row> = recs
            .into_iter()
            .map(|l| row_of(board, &typed, l, run.now_ms))
            .collect();
        if run.options.flag("json") {
            let launchers: Vec<Value> = rows.iter().map(Row::to_json).collect();
            as_json(
                run.out,
                &json!({ "launchers": launchers, "master": has_master(&typed) }),
            );
            return Ok(());
        }
        if rows.is_empty() {
            run.say(&format!("no launcher yet: coord launcher set {MASTER} --cmd \"<command that starts the main checkout's app>\" (a master launcher is required)."));
            return Ok(());
        }
        let mut lines = vec![HEAD.iter().map(|h| (*h).to_owned()).collect::<Vec<_>>()];
        lines.extend(rows.iter().map(|r| line(r, run.now_ms)));
        run.say(&table(&lines));
        if !has_master(&typed) {
            run.say(&format!("no master launcher: every project must have one, and coord build hand is refused without it. Set it: coord launcher set {MASTER} --cmd \"<command>\"."));
        }
        let waiting: Vec<String> = rows
            .iter()
            .filter(|r| truthy(&r.rec, "relink") && !truthy(&r.rec, "retiredAt"))
            .map(|r| display(&r.rec["name"]))
            .collect();
        if !waiting.is_empty() {
            run.say(&format!(
                "{}: no item yet (it closes with its item): coord launcher link <name> I-n.",
                waiting.join(", ")
            ));
        }
        let legacy: Vec<String> = rows
            .iter()
            .filter(|r| r.usage.as_ref().is_some_and(|u| u.legacy))
            .map(|r| display(&r.rec["name"]))
            .collect();
        if !legacy.is_empty() {
            run.say(&format!("{}: no --use-every yet (treated as {EVERY_LEGACY}): coord launcher set <name> --use-every <window>.", legacy.join(", ")));
        }
        Ok(())
    })
}
