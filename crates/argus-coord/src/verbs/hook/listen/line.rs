//! The line a heard wake prints on stderr, the text Claude Code wakes the model with:
//! `WAKE <thread>: <lines joined by "; ">[ (<n> wakes)]. Run: coord inbox`, or
//! `WAKE <thread>: see coord inbox` when the event data cannot be read.

use argus_board::json::{display, shown};
use argus_core::js;
use serde_json::Value;

fn lines_of(data: &Value) -> Option<String> {
    match data.get("lines") {
        None | Some(Value::Null) => Some(String::new()),
        Some(Value::Array(lines)) => Some(
            lines
                .iter()
                .map(|l| {
                    if l.is_null() {
                        String::new()
                    } else {
                        display(l)
                    }
                })
                .collect::<Vec<_>>()
                .join("; "),
        ),
        Some(_) => None,
    }
}

/// `count > 1` as JavaScript compares a number or a numeric text.
fn many(count: &Value) -> bool {
    match count {
        Value::Number(n) => n.as_f64().is_some_and(|n| n > 1.0),
        Value::String(s) => {
            let t = js::trim(s);
            let n = if t.is_empty() {
                Some(0.0)
            } else {
                t.parse::<f64>().ok()
            };
            n.is_some_and(|n| n > 1.0)
        }
        _ => false,
    }
}

fn line_of(data: &Value) -> Option<String> {
    if data.is_null() {
        return None;
    }
    let lines = lines_of(data)?;
    let count = data.get("count").unwrap_or(&Value::Null);
    let suffix = if many(count) {
        format!(" ({} wakes)", display(count))
    } else {
        String::new()
    };
    Some(format!(
        "WAKE {}: {lines}{suffix}. Run: coord inbox",
        shown(data.get("thread"))
    ))
}

/// The wake line of the event data `data`, for the listener of `thread`.
pub fn wake_line(data: &str, thread: &str) -> String {
    serde_json::from_str::<Value>(data)
        .ok()
        .and_then(|v| line_of(&v))
        .unwrap_or_else(|| format!("WAKE {thread}: see coord inbox"))
}
