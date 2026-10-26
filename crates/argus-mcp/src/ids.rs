//! Ids of the entries the owner posts through MCP.

use argus_core::clock::now_ms;

fn base36(n: u64) -> String {
    let digits = std::iter::successors(Some(n), |&rest| (rest >= 36).then_some(rest / 36))
        .map(|rest| char::from_digit(u32::try_from(rest % 36).unwrap_or(0), 36).unwrap_or('0'));
    digits.collect::<Vec<char>>().into_iter().rev().collect()
}

/// `O-<base36 milliseconds>-<4 hex digits>`.
#[must_use]
pub fn new_id() -> String {
    let mut bytes = [0u8; 2];
    getrandom::fill(&mut bytes).unwrap_or_default();
    format!(
        "O-{}-{:02x}{:02x}",
        base36(u64::try_from(now_ms()).unwrap_or(0)),
        bytes[0],
        bytes[1]
    )
}
