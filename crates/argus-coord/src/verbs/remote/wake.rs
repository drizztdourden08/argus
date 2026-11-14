//! `coord wake due [--json]`: the threads that hold a queued wake and have no listener, one line
//! each: board key, thread, wake count, oldest age and the Desktop session id (`local_...`) or `-`.
//! Any thread may read it; a relay uses it to know whom to resume.

use argus_core::clock::parse_ms;
use serde_json::{Value, json};

use super::support::link::{connect, list_of, take_sub};
use super::support::text::cell;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::out;
use crate::cli::route::Call;

/// An age as `45s`, `12m`, `3h` or `2d`.
fn age(iso: &str, now_ms: i64) -> String {
    let secs = parse_ms(iso).map_or(0, |then| ((now_ms - then) / 1000).max(0));
    match secs {
        0..=59 => format!("{secs}s"),
        60..=3599 => format!("{}m", secs / 60),
        3600..=172_799 => format!("{}h", secs / 3600),
        _ => format!("{}d", secs / 86_400),
    }
}

/// One thread as a line: `<board> <thread> <count> <oldest age> <local id or ->`.
fn line(due: &Value, now_ms: i64) -> String {
    let local = due["localId"].as_str().unwrap_or("-");
    format!(
        "{} {} {} {} {local}",
        cell(due.get("project")),
        cell(due.get("thread")),
        due["count"].as_u64().unwrap_or(0),
        age(due["oldest"].as_str().unwrap_or_default(), now_ms),
    )
}

fn due(call: &mut Call<'_>) -> Outcome {
    let answer = connect(call, false)?.get("/api/wakes/due")?;
    let threads = list_of(&answer, "due");
    if call.options.flag("json") {
        out::as_json(&mut call.inv.stdout, &json!(threads));
        return Ok(());
    }
    if threads.is_empty() {
        call.inv.say(&["no queued wakes without a listener."]);
        return Ok(());
    }
    let now = call.inv.clock.now_ms();
    let lines: Vec<String> = threads.iter().map(|t| line(t, now)).collect();
    call.inv.say(&[&lines.join("\n")]);
    Ok(())
}

/// `coord wake due`.
///
/// # Errors
/// A refusal, or the dashboard's own.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    let sub = take_sub(call);
    match sub.as_deref() {
        None | Some("due") => due(call),
        Some(other) => Err(CoordError::refuse(format!(
            "unknown wake command \"{other}\" (due)."
        ))),
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    fn entry(local: Option<&str>) -> Value {
        json!({
            "project": "argus",
            "thread": "lead",
            "count": 3,
            "oldest": "2027-01-15T07:59:00.000Z",
            "sessionId": "cli-1",
            "localId": local,
        })
    }

    #[test]
    fn a_line_names_board_thread_count_age_and_the_local_id() {
        let at = parse_ms("2027-01-15T08:00:00.000Z").expect("time");
        let text = line(&entry(Some("local_abc")), at);
        assert_eq!(text, "argus lead 3 1m local_abc");
    }

    #[test]
    fn a_thread_the_mapping_did_not_find_shows_a_dash() {
        let at = parse_ms("2027-01-15T11:00:00.000Z").expect("time");
        assert_eq!(line(&entry(None), at), "argus lead 3 3h -");
    }

    #[test]
    fn ages_step_from_seconds_to_days() {
        let then = "2027-01-15T08:00:00.000Z";
        let at = parse_ms(then).expect("time");
        assert_eq!(age(then, at + 45_000), "45s");
        assert_eq!(age(then, at + 3 * 86_400_000), "3d");
        assert_eq!(age(then, at - 5_000), "0s");
    }
}
