//! `coord activity [--limit n] [--kind k] [--level info|warn|error] [--json]`: what the dashboard
//! noticed and did (Argus > Activity), newest first, 30 lines by default.

use argus_board::json::to_number;
use serde_json::{Value, json};

use super::support::link::{connect, list_of};
use super::support::query::Query;
use super::support::text::{cell, int_within, say_all};
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::out;
use crate::cli::route::Call;

fn limit_of(call: &Call<'_>) -> Result<i64, CoordError> {
    let typed = call.options.json("limit");
    let n = typed.as_ref().map_or(30.0, |v| to_number(Some(v)));
    int_within(n, 1.0, 1000.0)
        .ok_or_else(|| CoordError::refuse("--limit takes a number from 1 to 1000."))
}

fn line(entry: &Value, now: i64) -> String {
    let mark = match entry["level"].as_str() {
        Some("info") => "    ",
        Some("warn") => "WARN",
        _ => "ERR ",
    };
    let ago = out::pad_end(&out::ago(entry["at"].as_str(), now), 8);
    let kind = out::pad_end(&cell(entry.get("kind")), 14);
    format!("{ago} {mark} {kind} {}", cell(entry.get("text")))
}

/// `coord activity`.
///
/// # Errors
/// A refusal, or the dashboard's own.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    let limit = limit_of(call)?;
    let mut query = Query::new().with("limit", &limit.to_string());
    for key in ["kind", "level"] {
        if let Some(value) = call.options.text(key) {
            query = query.with(key, value);
        }
    }
    let client = connect(call, false)?;
    let answer = client.get(&format!("/api/activity{}", query.suffix()))?;
    let activity = list_of(&answer, "activity");
    if call.options.flag("json") {
        out::as_json(&mut call.inv.stdout, &json!(activity));
    } else if activity.is_empty() {
        call.inv.say(&["no activity."]);
    } else {
        let now = call.inv.clock.now_ms();
        let lines: Vec<String> = activity.iter().map(|a| line(a, now)).collect();
        say_all(call.inv, &lines);
    }
    Ok(())
}
