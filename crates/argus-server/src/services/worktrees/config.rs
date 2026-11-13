//! `config.worktrees` as the service's options and the server's own timings.

use argus_worktrees::{AutoMode, Options, REMOVE_TIMEOUT, ScratchMode, ScratchRoot};
use serde_json::Value;

use crate::Services;
use crate::adapters::worktrees::{AutoSetting, extras_of};

/// When the server scans, apart from the service's own settings.
#[derive(Clone, Copy, Debug)]
pub(super) struct Timing {
    /// The first scan after the start.
    pub start_delay: u64,
    /// A scan after a change of rules waits for this long, so several changes make one.
    pub debounce: u64,
    /// Leftovers a scan had no time to measure are scanned again after this long.
    pub more: u64,
}

fn setting(svc: &Services) -> Value {
    svc.config()
        .extra
        .get("worktrees")
        .cloned()
        .unwrap_or(Value::Null)
}

/// A number of at least `min`, else the default.
fn number(setting: &Value, name: &str, default: u64, min: u64) -> u64 {
    setting[name]
        .as_u64()
        .filter(|n| *n >= min)
        .unwrap_or(default)
}

fn int(setting: &Value, name: &str, default: i64, min: u64) -> i64 {
    i64::try_from(number(
        setting,
        name,
        u64::try_from(default).unwrap_or(0),
        min,
    ))
    .unwrap_or(default)
}

/// One `worktrees.scratch` entry: a root, the include globs, the days untouched (7 by default) and the mode (`report`
/// unless it says `remove`). An entry with no root is dropped.
fn scratch_entry(entry: &Value) -> Option<ScratchRoot> {
    let root = entry["root"].as_str().filter(|r| !r.trim().is_empty())?;
    Some(ScratchRoot {
        root: root.to_string(),
        include: entry["include"]
            .as_array()
            .map(|globs| {
                globs
                    .iter()
                    .filter_map(|g| g.as_str().map(ToString::to_string))
                    .collect()
            })
            .unwrap_or_default(),
        min_age_days: entry["minAgeDays"]
            .as_f64()
            .filter(|d| *d > 0.0)
            .unwrap_or(7.0),
        mode: if entry["mode"] == "remove" {
            ScratchMode::Remove
        } else {
            ScratchMode::Report
        },
    })
}

/// `worktrees.scratch`: the scratch roots, none by default.
fn scratch_roots(setting: &Value) -> Vec<ScratchRoot> {
    setting["scratch"]
        .as_array()
        .map(|list| list.iter().filter_map(scratch_entry).collect())
        .unwrap_or_default()
}

/// `worktrees.keep`: path globs of worktrees the automatic hygiene never touches, none by default.
fn keep_globs(setting: &Value) -> Vec<String> {
    setting["keep"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|g| g.as_str().map(str::trim))
                .filter(|g| !g.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn timing_of(svc: &Services) -> Timing {
    let s = setting(svc);
    Timing {
        start_delay: number(&s, "startDelayMs", 5000, 0),
        debounce: number(&s, "debounceMs", 800, 0),
        more: number(&s, "leftoverMoreMs", 20_000, 0),
    }
}

/// `worktrees.autoRules`: `report` (the default), `on` or `off`.
fn auto_mode(setting: &Value) -> AutoMode {
    setting["autoRules"]
        .as_str()
        .map_or(AutoMode::Report, AutoMode::parse)
}

/// The automatic repo rules: their mode and `worktrees.extraRepos`, whose owner is the portfolio board by default.
pub(super) fn auto_setting(svc: &Services) -> AutoSetting {
    let s = setting(svc);
    let owner = svc.boards().overlord().unwrap_or_default();
    AutoSetting {
        mode: auto_mode(&s),
        extras: extras_of(&s["extraRepos"], &owner),
    }
}

/// The service's settings: scan and repeat periods, the stale and leftover limits, the folder
/// of the backups.
pub(super) fn options_of(svc: &Services) -> Options {
    let s = setting(svc);
    let base = Options::default();
    let mut options = Options {
        scan_ms: int(&s, "scanMs", base.scan_ms, 1000),
        repeat_ms: int(&s, "repeatMs", base.repeat_ms, 1),
        live_ms: int(&s, "liveMs", base.live_ms, 1000),
        many_at: usize::try_from(number(&s, "manyAt", 12, 1)).unwrap_or(12),
        stale_days: u32::try_from(number(&s, "staleDays", 3, 1)).unwrap_or(3),
        remeasure_ms: int(&s, "leftoverRemeasureMs", base.remeasure_ms, 0),
        budget_ms: int(&s, "leftoverBudgetMs", base.budget_ms, 0),
        backup_dir: svc
            .paths
            .home
            .join("worktree-backups")
            .to_string_lossy()
            .into_owned(),
        run_requests: !svc.shadow,
        hygiene: !svc.shadow && s["hygiene"] != false && s["release"] != false,
        auto_rules: auto_mode(&s),
        detach_debounce_ms: int(&s, "detachDebounceMs", base.detach_debounce_ms, 0),
        report_after_ms: int(&s, "reportAfterMs", base.report_after_ms, 0),
        scratch: scratch_roots(&s),
        scratch_every_ms: int(&s, "scratchEveryMs", base.scratch_every_ms, 1000),
        keep: keep_globs(&s),
        ..base
    };
    if let Some(days) = s["leftoverAgeDays"].as_f64().filter(|d| *d >= 0.0) {
        options.leftover_age_days = days;
    }
    options.measure_bounds.max_entries = number(
        &s,
        "leftoverMaxEntries",
        options.measure_bounds.max_entries,
        1,
    );
    options.measure_bounds.max_ms = int(&s, "leftoverMaxMs", options.measure_bounds.max_ms, 1);
    options
}

/// How many git processes at once, how long a read may take, and how long `git worktree remove` may take.
pub(super) fn git_limits(svc: &Services) -> (usize, u64, u64) {
    let s = setting(svc);
    let remove_ms = u64::try_from(REMOVE_TIMEOUT.as_millis()).unwrap_or(900_000);
    (
        usize::try_from(number(&s, "maxProcs", 2, 1)).unwrap_or(2),
        number(&s, "gitTimeoutMs", 10_000, 100),
        number(&s, "removeTimeoutMs", remove_ms, 1000),
    )
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn keep_reads_the_trimmed_non_empty_strings_and_defaults_to_none() {
        let s = json!({"keep": ["X:/geo-survivors-v*", " X:/geo-survivors-play* ", "", 3]});
        assert_eq!(
            keep_globs(&s),
            vec!["X:/geo-survivors-v*", "X:/geo-survivors-play*"]
        );
        assert_eq!(keep_globs(&json!({})).len(), 0);
        assert_eq!(keep_globs(&Value::Null).len(), 0);
    }
}
