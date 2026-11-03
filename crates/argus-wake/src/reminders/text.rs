//! The one line the Overlord hears about everything still open.

use argus_core::clock::age_text;

use super::OpenNotice;
use crate::faults::Fault;
use crate::time::ms_or_zero;

fn faults_part(now: i64, faults: &[&Fault]) -> Option<String> {
    let oldest = faults
        .iter()
        .map(|f| ms_or_zero(Some(f.since.as_str())))
        .min()?;
    let noun = if faults.len() == 1 { "fault" } else { "faults" };
    Some(format!(
        "{} {noun} open, oldest {}",
        faults.len(),
        age_text(now - oldest)
    ))
}

fn notice_parts(notices: &[OpenNotice]) -> Vec<String> {
    let mut counts: Vec<(&str, usize)> = Vec::new();
    for notice in notices {
        match counts
            .iter_mut()
            .find(|(reason, _)| *reason == notice.reason)
        {
            Some((_, count)) => *count += 1,
            None => counts.push((notice.reason.as_str(), 1)),
        }
    }
    counts
        .into_iter()
        .map(|(reason, count)| format!("{count} {reason} open"))
        .collect()
}

/// `2 faults open, oldest 3h; 1 worktree-stale open`, or `None` with nothing open.
#[must_use]
pub fn open_line(now: i64, faults: &[&Fault], notices: &[OpenNotice]) -> Option<String> {
    let mut parts: Vec<String> = faults_part(now, faults).into_iter().collect();
    parts.extend(notice_parts(notices));
    (!parts.is_empty()).then(|| parts.join("; "))
}
