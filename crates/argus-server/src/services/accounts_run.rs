//! The `run` of `GET /api/accounts`: where the current switch stands, as one phase a stepper can
//! show, read from the `switch` doc the web already follows. The switch itself waits for the
//! handovers (or a confirmed limit) as it always did; this only names the step.
//!
//! The phases are `idle`, `handover-asked`, `handover-done`, `limit-confirmed`, `switching`,
//! `switched` and `failed`. A countdown with no handover and no confirmed limit is `idle`.

use argus_core::clock::parse_ms;
use serde_json::{Value, json};

/// Whether every lead of the handover has an answer: ready, cut off or failed.
fn handovers_in(doc: &Value) -> bool {
    let threads = doc["threads"].as_array().map_or(&[][..], Vec::as_slice);
    !threads.is_empty()
        && threads
            .iter()
            .all(|t| !matches!(t["state"].as_str(), Some("found" | "asked")))
}

/// The phase of the run for a `switch` doc phase.
fn phase_of(doc: &Value, limit_confirmed: bool) -> &'static str {
    let waiting = |in_hand: bool| {
        if limit_confirmed {
            "limit-confirmed"
        } else if in_hand {
            "handover-done"
        } else {
            "handover-asked"
        }
    };
    match doc["phase"].as_str().unwrap_or("idle") {
        "failed" => "failed",
        "done" | "verifying" => "switched",
        "switching" | "sync" => "switching",
        "handover" => waiting(handovers_in(doc)),
        "countdown" if handovers_in(doc) => "handover-done",
        "countdown" if limit_confirmed => "limit-confirmed",
        _ => "idle",
    }
}

/// The time of the last step of the log, epoch milliseconds.
fn last_step_ms(doc: &Value) -> Option<i64> {
    let steps = doc["steps"].as_array()?;
    steps.iter().rev().find_map(|s| parse_ms(s["at"].as_str()?))
}

/// `{ "phase": ..., "atMs": number | null }` for the `switch` doc shown now, and whether the
/// account being left has a confirmed limit.
pub(crate) fn run_shown(doc: Option<&Value>, limit_confirmed: bool) -> Value {
    let Some(doc) = doc else {
        return json!({ "phase": "idle", "atMs": null });
    };
    let phase = phase_of(doc, limit_confirmed);
    let at = if phase == "idle" {
        None
    } else {
        last_step_ms(doc)
    };
    json!({ "phase": phase, "atMs": at })
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    fn doc(phase: &str, states: &[&str]) -> Value {
        let threads: Vec<Value> = states.iter().map(|s| json!({ "state": s })).collect();
        json!({
            "phase": phase,
            "threads": threads,
            "steps": [{ "at": "2026-11-23T10:00:00.000Z" }, { "at": "2026-11-23T10:00:01.000Z" }],
        })
    }

    fn phase(doc: &Value, confirmed: bool) -> String {
        run_shown(Some(doc), confirmed)["phase"]
            .as_str()
            .unwrap_or_default()
            .to_owned()
    }

    #[test]
    fn no_doc_is_idle() {
        assert_eq!(
            run_shown(None, false),
            json!({ "phase": "idle", "atMs": null })
        );
        assert_eq!(phase(&doc("cancelled", &[]), false), "idle");
    }

    #[test]
    fn the_handover_asks_then_is_done() {
        assert_eq!(
            phase(&doc("handover", &["asked", "ready"]), false),
            "handover-asked"
        );
        assert_eq!(
            phase(&doc("handover", &["ready", "skipped"]), false),
            "handover-done"
        );
        assert_eq!(phase(&doc("countdown", &["ready"]), false), "handover-done");
    }

    #[test]
    fn a_confirmed_limit_outranks_the_wait() {
        assert_eq!(phase(&doc("handover", &["asked"]), true), "limit-confirmed");
        assert_eq!(phase(&doc("countdown", &[]), true), "limit-confirmed");
        assert_eq!(phase(&doc("countdown", &[]), false), "idle");
    }

    #[test]
    fn the_switch_and_its_end() {
        assert_eq!(phase(&doc("sync", &[]), false), "switching");
        assert_eq!(phase(&doc("switching", &[]), false), "switching");
        assert_eq!(phase(&doc("verifying", &[]), false), "switched");
        assert_eq!(phase(&doc("done", &[]), false), "switched");
        assert_eq!(phase(&doc("failed", &[]), false), "failed");
    }

    #[test]
    fn the_time_is_the_last_step() {
        let at = run_shown(Some(&doc("done", &[])), false)["atMs"].as_i64();
        assert_eq!(at, parse_ms("2026-11-23T10:00:01.000Z"));
        assert!(at.is_some());
    }
}
