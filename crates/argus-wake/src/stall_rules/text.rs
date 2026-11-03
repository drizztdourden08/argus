//! The words of the stall lines the Overlord reads.

const MIN_MS: i64 = 60_000;
const SHOWN_IDS: usize = 5;

/// Whole minutes of an age in milliseconds.
fn minutes(age_ms: i64) -> i64 {
    age_ms / MIN_MS
}

/// `stall: D-11 open 30 min, rotp I-112 still todo` (the item as the owner reads its id)
pub(super) fn unstarted(dep: &str, age_ms: i64, project: &str, item: &str) -> String {
    format!(
        "stall: {dep} open {} min, {project} {item} still todo",
        minutes(age_ms)
    )
}

/// `stall: rotp idle 12 min, 3 open items without a blocker (K-4, B-7, K-9)`; at most 5 shown ids, then `, ...`.
pub(super) fn idle(project: &str, age_ms: i64, ids: &[String]) -> String {
    let shown = ids
        .iter()
        .take(SHOWN_IDS)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    let more = if ids.len() > SHOWN_IDS { ", ..." } else { "" };
    format!(
        "stall: {project} idle {} min, {} open items without a blocker ({shown}{more})",
        minutes(age_ms),
        ids.len()
    )
}

/// `stall: rotp/K-12 doing 60 min, no agent` (the item as the owner reads its id)
pub(super) fn silent(project: &str, item: &str, age_ms: i64) -> String {
    format!(
        "stall: {project}/{item} doing {} min, no agent",
        minutes(age_ms)
    )
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::{idle, silent, unstarted};

    fn ids(count: usize) -> Vec<String> {
        (1..=count).map(|n| format!("I-{n}")).collect()
    }

    #[test]
    fn the_lines_read_as_the_spec_words_with_whole_minutes() {
        assert_eq!(
            unstarted("D-11", 31 * 60_000 + 59_000, "rotp", "I-112"),
            "stall: D-11 open 31 min, rotp I-112 still todo"
        );
        assert_eq!(
            silent("rotp", "K-12", 60 * 60_000),
            "stall: rotp/K-12 doing 60 min, no agent"
        );
        assert_eq!(
            idle("rotp", 12 * 60_000, &ids(3)),
            "stall: rotp idle 12 min, 3 open items without a blocker (I-1, I-2, I-3)"
        );
    }

    #[test]
    fn the_idle_line_shows_five_ids_then_dots() {
        assert_eq!(
            idle("rotp", 600_000, &ids(5)),
            "stall: rotp idle 10 min, 5 open items without a blocker (I-1, I-2, I-3, I-4, I-5)"
        );
        assert_eq!(
            idle("rotp", 600_000, &ids(7)),
            "stall: rotp idle 10 min, 7 open items without a blocker (I-1, I-2, I-3, I-4, I-5, ...)"
        );
    }
}
