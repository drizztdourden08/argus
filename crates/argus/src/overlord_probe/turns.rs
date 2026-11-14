//! One turn of the conversation, and the cost facts around it.

use std::time::Duration;

use argus_overlord::protocol::{Event, Outcome, encode_user_message};

use super::child::Child;
use super::probe::Probe;

/// How long one turn may take.
pub const TURN_TIMEOUT: Duration = Duration::from_secs(120);

/// Sends `text`, waits for its `result` and prints the turn facts under `step`.
pub async fn turn(
    probe: &mut Probe,
    child: &mut Child,
    step: &'static str,
    text: &str,
) -> Result<Outcome, String> {
    probe.step = step;
    child.send(&encode_user_message(text)).await?;
    wait_result(probe, child, step).await
}

/// Waits for the next `result` and prints its facts under `step`.
pub async fn wait_result(
    probe: &mut Probe,
    child: &mut Child,
    step: &'static str,
) -> Result<Outcome, String> {
    probe.step = step;
    let waited = tokio::time::timeout(TURN_TIMEOUT, next_result(probe, child)).await;
    let outcome = waited.map_err(|_| format!("timeout (120 s) in step {step}"))??;
    print_outcome(probe, step, &outcome);
    Ok(outcome)
}

async fn next_result(probe: &mut Probe, child: &mut Child) -> Result<Outcome, String> {
    loop {
        if let Event::Result(outcome) = child.next_event(probe).await? {
            return Ok(outcome);
        }
    }
}

fn print_outcome(probe: &mut Probe, step: &str, outcome: &Outcome) {
    probe.fact(
        &format!("{step} total_cost_usd"),
        &show(outcome.total_cost_usd),
    );
    probe.fact(&format!("{step} usage"), &format!("{:?}", outcome.usage));
    probe.fact(&format!("{step} is_error"), &outcome.is_error.to_string());
    probe.fact(&format!("{step} subtype"), &outcome.subtype);
    probe.fact(
        &format!("{step} num_turns"),
        &format!("{:?}", outcome.num_turns),
    );
}

fn show(cost: Option<f64>) -> String {
    cost.map_or_else(|| "none".to_owned(), |c| format!("{c:.6}"))
}

/// `yes`, `no` or `unclear`: does turn 2's total carry turn 1's cost?
///
/// Per turn totals would sit near the first one (a ratio near 1), cumulative totals near the
/// sum (a ratio well above 1).
pub fn cumulative(first: Option<f64>, second: Option<f64>) -> &'static str {
    match (first, second) {
        (Some(a), Some(b)) if a > 0.0 && b / a >= 1.2 => "yes",
        (Some(a), Some(b)) if a > 0.0 && b / a <= 1.1 => "no",
        _ => "unclear",
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::cumulative;

    #[test]
    fn the_ratio_decides() {
        assert_eq!(cumulative(Some(0.01), Some(0.02)), "yes");
        assert_eq!(cumulative(Some(0.01), Some(0.01)), "no");
        assert_eq!(cumulative(Some(0.01), Some(0.0114)), "unclear");
        assert_eq!(cumulative(None, Some(0.01)), "unclear");
    }
}
