//! `coord launcher show <name> [--json]`: one launcher, with the bundles that link it.

use argus_board::json::{doc, shown, truthy};
use argus_launchers::launcher::{Platform, shell_of};
use serde_json::{Value, json};

use super::name::name_arg;
use super::row::{Row, row_of};
use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::out::{ago, as_json, pad_end};
use crate::verbs::agent::base::read;
use crate::verbs::agent::base::show::cell;
use crate::verbs::agent::rules::typed_board;

/// `key      value`, or nothing for an empty value.
fn line(key: &str, value: &str) -> Option<String> {
    (!value.is_empty()).then(|| format!("{} {value}", pad_end(key, 8)))
}

fn header(name: &str, rec: &Value, now_ms: i64) -> String {
    let title = if truthy(rec, "title") {
        format!(": {}", cell(&rec["title"]))
    } else {
        String::new()
    };
    let current = if truthy(rec, "current") {
        " (current)"
    } else {
        ""
    };
    let retired = if truthy(rec, "retiredAt") {
        let note = if truthy(rec, "retireNote") {
            format!(": {}", cell(&rec["retireNote"]))
        } else {
            String::new()
        };
        format!(
            " (retired {} by {}{note})",
            ago(rec["retiredAt"].as_str(), now_ms),
            shown(rec.get("retiredBy"))
        )
    } else {
        String::new()
    };
    format!("{name}{title}{current}{retired}")
}

/// The `window` line of a standalone launcher: how often it is used, when it was last, who kept it.
fn window(row: &Row, now_ms: i64) -> Option<String> {
    let rec = &row.rec;
    let usage = row
        .usage
        .as_ref()
        .filter(|_| row.standalone && !truthy(rec, "retiredAt"))?;
    let default = if usage.legacy {
        " (default: set --use-every)"
    } else {
        ""
    };
    let stale = if usage.stale { " -- STALE" } else { "" };
    let kept = if truthy(rec, "keptAt") {
        let until = if truthy(rec, "keptUntil") {
            format!(" until {}", cell(&rec["keptUntil"]))
        } else {
            String::new()
        };
        let note = if truthy(rec, "keptNote") {
            format!(" ({})", cell(&rec["keptNote"]))
        } else {
            String::new()
        };
        format!(
            "; kept by {} {}{until}{note}",
            shown(rec.get("keptBy")),
            ago(rec["keptAt"].as_str(), now_ms)
        )
    } else {
        String::new()
    };
    let text = format!(
        "used every {}{default}; last used {}{stale}{kept}",
        usage.use_every,
        ago(Some(&usage.last_used_at), now_ms)
    );
    line("window", &text)
}

/// The `master` line: only the master build has it.
fn master_text(rec: &Value) -> String {
    if truthy(rec, "master") {
        "the master build: it needs no item and never closes with one".to_owned()
    } else {
        String::new()
    }
}

/// The `link` line of a live build the migration could not match, and of a closed one.
fn link_text(name: &str, rec: &Value) -> String {
    if truthy(rec, "retiredByItem") {
        return format!("closed with {}", cell(&rec["retiredByItem"]));
    }
    if truthy(rec, "relink") {
        return format!("no item yet: coord launcher link {name} I-n");
    }
    String::new()
}

fn lines(name: &str, row: &Row, now_ms: i64) -> Vec<String> {
    let rec = &row.rec;
    let play = row.play.clone().unwrap_or_else(|| {
        if truthy(rec, "retiredAt") {
            "none (retired)".to_owned()
        } else {
            "none (the command spans several lines, or the run name is taken)".to_owned()
        }
    });
    let set = format!(
        "{} {} (made by {} {})",
        shown(rec.get("by")),
        ago(rec["updatedAt"].as_str(), now_ms),
        shown(rec.get("createdBy")),
        ago(rec["createdAt"].as_str(), now_ms)
    );
    let joined = row.joined.as_ref().map_or_else(String::new, |b| {
        format!("for bundle {b}: it goes with the bundle, no usage window")
    });
    let runs = format!(
        "{}, from the folder above, hidden, stdin a pipe",
        shell_of(Platform::current())
    );
    [
        Some(header(name, rec, now_ms)),
        line("command", &cell(&rec["cmd"])),
        line("runs", &runs),
        line("cwd", &cell(&rec["cwd"])),
        line("kind", &cell(&rec["kind"])),
        line("branch", &cell(&rec["branch"])),
        line("commit", &cell(&rec["commit"])),
        line("note", &cell(&rec["note"])),
        line("set", &set),
        line("play", &play),
        line("bundles", &row.bundles.join(", ")),
        line("for", &cell(&rec["for"])),
        line("master", &master_text(rec)),
        line("link", &link_text(name, rec)),
        line("joined", &joined),
        window(row, now_ms),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// `coord launcher show`.
///
/// # Errors
/// No such launcher.
pub fn run(call: &mut Call<'_>) -> Outcome {
    let name = name_arg(&call.positional, "show")?;
    read(call, |board, run| {
        let Some(rec) = doc(board, "launchers", &name) else {
            return Err(CoordError::refuse(format!("no launcher \"{name}\".")));
        };
        let typed = typed_board(board)?;
        let row = row_of(board, &typed, rec, run.now_ms);
        if run.options.flag("json") {
            as_json(run.out, &json!({ "launcher": row.to_json() }));
            return Ok(());
        }
        run.say(&lines(&name, &row, run.now_ms).join("\n"));
        Ok(())
    })
}
