//! The targets of a `finished` or `leftovers` request, from the listing right after the scan. The guards still decide each.

use super::MAX_FINISHED;
use super::record::{Mode, RequestRec, SkippedSummary, Target};
use super::text::idle_text;
use crate::board::Caller;
use crate::doc::{Status, WorktreeDoc};
use crate::error::Result;
use crate::request::RemoveInput;
use crate::service::Service;
use argus_core::clock::parse_ms;

/// The targets of a request, what was left out and why, and a word about an empty answer.
pub(super) struct Resolved {
    pub targets: Vec<Target>,
    pub skipped: Option<SkippedSummary>,
    pub note: Option<String>,
}

fn target_of(doc: &WorktreeDoc) -> Target {
    Target {
        id: doc.id.clone(),
        label: doc.path.clone(),
    }
}

/// Merged, or stale and clean.
fn is_finished(doc: &WorktreeDoc) -> bool {
    doc.status == Status::Merged || (doc.status == Status::Stale && doc.dirty == 0)
}

/// Clean, nothing ahead, and merged (a branch with no upstream reports ahead 0 even with its own commits, so
/// `merged` is the real ancestor check), with its newest change older than the idle time.
fn is_idle(doc: &WorktreeDoc, idle_ms: i64, now_ms: i64) -> bool {
    let touched = doc.last_touched_at.as_deref().and_then(parse_ms);
    doc.dirty == 0
        && doc.ahead == 0
        && doc.merged
        && doc.status != Status::Missing
        && touched.is_some_and(|t| now_ms - t > idle_ms)
}

fn nothing_finished(idle_minutes: Option<f64>) -> String {
    match idle_minutes.filter(|m| *m != 0.0) {
        Some(m) => format!(
            "no finished worktree (merged, or stale and clean) or idle clean worktree older than {} in the listing",
            idle_text(m)
        ),
        None => "no finished worktree (merged, or stale and clean) in the listing".to_string(),
    }
}

fn finished_targets(docs: &[WorktreeDoc], idle_minutes: Option<f64>, now_ms: i64) -> Resolved {
    #[allow(clippy::cast_possible_truncation)]
    let idle_ms = idle_minutes.map_or(0, |m| (m * 60_000.0) as i64);
    let wanted = |d: &&WorktreeDoc| {
        d.leftover.is_none()
            && !d.main
            && d.locked.is_none()
            && (is_finished(d) || (idle_ms > 0 && is_idle(d, idle_ms, now_ms)))
    };
    let finished: Vec<&WorktreeDoc> = docs.iter().filter(wanted).collect();
    let note = if finished.len() > MAX_FINISHED {
        Some(format!(
            "only the first {MAX_FINISHED} of {} finished worktrees; ask again for the rest",
            finished.len()
        ))
    } else if finished.is_empty() {
        Some(nothing_finished(idle_minutes))
    } else {
        None
    };
    Resolved {
        targets: finished
            .into_iter()
            .take(MAX_FINISHED)
            .map(target_of)
            .collect(),
        skipped: None,
        note,
    }
}

/// `1`, `0.25`, `0.33`: the days with at most two decimals, as a number prints.
fn days_label(days: f64) -> String {
    let fixed = format!("{days:.2}");
    if fixed.contains('.') {
        fixed
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        fixed
    }
}

/// Why a leftover row is left out of the request, or none when it is old enough.
fn left_out(doc: &WorktreeDoc, min_days: f64) -> Option<String> {
    let leftover = doc.leftover.as_ref()?;
    if leftover.pending {
        return Some("not measured yet".to_string());
    }
    #[allow(clippy::cast_precision_loss)]
    let young = leftover.age_days.is_none_or(|a| (a as f64) < min_days);
    (!leftover.made && young).then(|| {
        let s = if (min_days - 1.0).abs() < f64::EPSILON {
            ""
        } else {
            "s"
        };
        format!("younger than {} day{s}", days_label(min_days))
    })
}

impl Service {
    fn leftover_targets(&self, rec: &RequestRec, docs: &[WorktreeDoc]) -> Result<Resolved> {
        let mut asked = RemoveInput::new(&rec.key, "", Caller::Thread(rec.by.clone()));
        asked.older_than_days = rec.older_than_days;
        asked.reason.clone_from(&rec.reason);
        let min_days = self.age_limit(&asked)?;
        let rows: Vec<&WorktreeDoc> = docs
            .iter()
            .filter(|d| d.status == Status::Leftover && d.leftover.is_some())
            .collect();
        let mut reasons: Vec<(String, usize)> = Vec::new();
        let mut take: Vec<Target> = Vec::new();
        for doc in &rows {
            match left_out(doc, min_days) {
                Some(why) => match reasons.iter_mut().find(|(r, _)| *r == why) {
                    Some((_, n)) => *n += 1,
                    None => reasons.push((why, 1)),
                },
                None => take.push(target_of(doc)),
            }
        }
        let count = reasons.iter().map(|(_, n)| n).sum();
        let skipped = (count > 0).then(|| SkippedSummary {
            count,
            reasons: reasons.iter().map(|(r, n)| format!("{n} {r}")).collect(),
        });
        let note = take.is_empty().then(|| {
            if rows.is_empty() {
                "no leftover folder in the listing".to_string()
            } else {
                "no leftover folder old enough".to_string()
            }
        });
        take.truncate(MAX_FINISHED);
        Ok(Resolved {
            targets: take,
            skipped,
            note,
        })
    }

    /// The targets of a request that named none, from the listing now.
    pub(super) fn resolve_request(&self, rec: &RequestRec) -> Result<Resolved> {
        let docs = self.list(&rec.key);
        match rec.mode {
            Mode::Finished => Ok(finished_targets(&docs, rec.idle_minutes, self.now())),
            _ => self.leftover_targets(rec, &docs),
        }
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::days_label;

    #[test]
    fn days_print_with_at_most_two_decimals() {
        let seen: Vec<String> = [1.0, 0.25, 10.0, 1.0 / 3.0, 0.5].map(days_label).to_vec();
        assert_eq!(seen, ["1", "0.25", "10", "0.33", "0.5"]);
    }
}
