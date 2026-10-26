//! `coord tick` (`verbs/tick.mjs`): the coordinator's manual check, never run on a schedule
//! (waking is the dashboard's wake stream). It only reads the board and prints JSON: `wake` lists
//! the idle threads holding messages no hook could hand over, `attention` what the coordinator
//! itself should look at. `coord tick woke <thread>...` records that those threads were messaged.

use argus_board::Board;
use argus_board::json::{display, nullish, section, set, truthy};
use argus_core::clock::now_iso;
use serde_json::{Value, json};

use super::docs::{age_ms, all, id_of, js_number, list};
use super::frame::{Frame, with_frame};
use crate::cli::exit::Outcome;
use crate::cli::out;
use crate::cli::route::Call;

const WAKE_AFTER_MS: f64 = 120_000.0;
const LONG_MS: f64 = 7_200_000.0;
const UNNAMED_MS: f64 = 600_000.0;
const HOUR_MS: f64 = 3_600_000.0;
const MINUTE_MS: f64 = 60_000.0;

/// `Math.round` of a non-negative number, as a template literal prints it.
fn rounded(n: f64) -> String {
    js_number((n + 0.5).floor())
}

fn to_wake(board: &Board, now_ms: i64) -> Vec<Value> {
    all(board, "threads")
        .into_iter()
        .filter(|t| !["done", "dispatched", "archived"].iter().any(|s| t["status"] == *s))
        .filter_map(|t| {
            let id = id_of(t);
            let waiting = list(&board["inbox"][id.as_str()])
                .iter()
                .filter(|m| {
                    !truthy(m, "deliveredAt")
                        && !truthy(m, "wokeAt")
                        && age_ms(&m["ts"], now_ms) > WAKE_AFTER_MS
                })
                .count();
            (waiting > 0).then(|| {
                json!({"thread": id, "session": t.get("session").filter(|s| !s.is_null()).cloned().unwrap_or(Value::Null), "count": waiting})
            })
        })
        .collect()
}

fn thread_notes(t: &Value, now_ms: i64) -> Vec<String> {
    let id = id_of(t);
    let mut out = Vec::new();
    if ["done", "dispatched", "paused", "archived"]
        .iter()
        .any(|s| t["status"] == *s)
    {
        return out;
    }
    if t["status"] == "blocked" || truthy(&t["flags"], "blocked") {
        out.push(format!(
            "{id} is blocked: {}",
            nullish(t, "note").unwrap_or_default()
        ));
    }
    let silent = age_ms(&t["lastSeen"], now_ms);
    if silent > LONG_MS {
        out.push(format!("{id} silent for {}h", rounded(silent / HOUR_MS)));
    }
    let placeholder = truthy(t, "placeholder");
    let unnamed = age_ms(&t["registeredAt"], now_ms);
    if placeholder && unnamed > UNNAMED_MS {
        let link = nullish(t, "link").unwrap_or_else(|| "no link".to_owned());
        out.push(format!(
            "{id} is a session that has not named itself for {}m: open it ({link}) or message it to register",
            rounded(unnamed / MINUTE_MS)
        ));
    } else if !truthy(t, "session") && !placeholder {
        out.push(format!("{id} has no session title, it cannot be woken"));
    }
    out
}

fn attention_of(board: &Board, now_ms: i64) -> Vec<Value> {
    let mut notes: Vec<String> = all(board, "threads")
        .into_iter()
        .flat_map(|t| thread_notes(t, now_ms))
        .collect();
    for l in section(board, "locks").values() {
        let queued = list(&l["queue"]).len();
        let held = age_ms(&l["since"], now_ms);
        if truthy(l, "holder") && queued > 0 && held > LONG_MS {
            notes.push(format!(
                "{} held by {} for {}h, {queued} waiting",
                display(&l["resource"]),
                display(&l["holder"]),
                rounded(held / HOUR_MS)
            ));
        }
    }
    notes.into_iter().map(Value::from).collect()
}

fn tick(frame: &mut Frame<'_>) -> Outcome {
    if frame.options.get("dir").is_some() {
        out::say(
            &mut *frame.err,
            &["coord: --dir is ignored; the dashboard syncs the board itself."],
        );
    }
    let board = frame.read_board()?;
    let now = frame.now_ms;
    frame.json(&json!({"wake": to_wake(&board, now), "attention": attention_of(&board, now)}));
    Ok(())
}

fn woke(frame: &mut Frame<'_>) -> Outcome {
    let threads: Vec<String> = frame.args.iter().skip(1).cloned().collect();
    frame.mutate(false, |board, fr| {
        for id in &threads {
            let messages = board
                .get_mut("inbox")
                .and_then(|inbox| inbox.get_mut(id.as_str()))
                .and_then(Value::as_array_mut);
            for m in messages.into_iter().flatten() {
                if !truthy(m, "deliveredAt") {
                    set(m, "wokeAt", now_iso());
                }
            }
        }
        let named = if threads.is_empty() {
            "nobody".to_owned()
        } else {
            threads.join(", ")
        };
        fr.say(&[&format!("woke: {named}")]);
        Ok(())
    })
}

/// The verb `tick` and `tick woke <thread>...`.
///
/// # Errors
/// Refused when there is no board.
pub fn run(call: &mut Call<'_>) -> Outcome {
    with_frame(call, |frame| {
        if frame.arg(0) == Some("woke") {
            woke(frame)
        } else {
            tick(frame)
        }
    })
}
