//! One poll pass over all accounts, the burn rate and the poll interval.

use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::time::Duration;

use argus_core::clock::iso_from_ms;

use super::UNAUTHORIZED;
use super::fallback::latest_sample;
use crate::config::AccountsConfig;
use crate::ports::{Credentials, UsageSource};
use crate::types::{Account, Usage, UsageLimit};

/// The window of samples the burn rate is worked out from.
const BURN_WINDOW_MS: i64 = 20 * 60_000;
const FALLBACK_ERROR: &str = "live read failed, showing Desktop log";

/// What one poll found for one account.
#[derive(Debug, Clone, PartialEq)]
pub struct PollResult {
    /// The account key.
    pub key: String,
    /// The usage read, or the Desktop log's when the live read failed (with `error` set).
    pub usage: Usage,
    /// Percent per minute of the binding limit over the last minutes; `None` when unknown.
    pub burn_per_min: Option<f64>,
    /// When the CLI login expires, epoch milliseconds.
    pub expires_at: Option<i64>,
    /// Whether the account folder holds a login the endpoint accepted or at least gave a token.
    pub logged_in: bool,
    /// Why the live read failed, for the log; never holds a token.
    pub failure: Option<String>,
}

/// The limit nearest its end: the highest percent among the active limits, or among all when
/// none is flagged active.
#[must_use]
pub fn binding_limit(usage: &Usage) -> Option<&UsageLimit> {
    let any_active = usage.limits.iter().any(|l| l.is_active);
    usage
        .limits
        .iter()
        .filter(|l| l.is_active || !any_active)
        .max_by(|a, b| a.percent.total_cmp(&b.percent))
}

/// The recent percents of each account, kept in memory.
#[derive(Debug, Default)]
pub struct BurnHistory {
    samples: HashMap<String, VecDeque<(i64, f64)>>,
}

impl BurnHistory {
    /// Notes a percent. A drop (the limit reset) starts the account's history over.
    pub fn record(&mut self, key: &str, now_ms: i64, percent: f64) {
        let queue = self.samples.entry(key.to_owned()).or_default();
        if queue.back().is_some_and(|(_, last)| percent < *last) {
            queue.clear();
        }
        queue.push_back((now_ms, percent));
        while queue
            .front()
            .is_some_and(|(t, _)| now_ms - *t > BURN_WINDOW_MS)
        {
            queue.pop_front();
        }
    }

    /// Percent per minute across the kept samples; `None` with fewer than two.
    #[must_use]
    pub fn per_min(&self, key: &str) -> Option<f64> {
        let queue = self.samples.get(key)?;
        let (t0, p0) = *queue.front()?;
        let (t1, p1) = *queue.back()?;
        #[allow(clippy::cast_precision_loss)] // a span of minutes
        let minutes = (t1 - t0) as f64 / 60_000.0;
        (minutes > 0.0).then(|| (p1 - p0) / minutes)
    }
}

/// What a poll pass needs from the outside.
pub struct Poll<'a> {
    /// The usage endpoint.
    pub source: &'a dyn UsageSource,
    /// The CLI logins.
    pub creds: &'a dyn Credentials,
    /// The Desktop app folder, for the log fallback.
    pub app: Option<&'a Path>,
}

/// Reads every account's usage once.
pub fn poll_all(
    accounts: &[Account],
    poll: &Poll<'_>,
    burn: &mut BurnHistory,
    now_ms: i64,
) -> Vec<PollResult> {
    accounts
        .iter()
        .map(|account| poll_one(account, poll, burn, now_ms))
        .collect()
}

fn poll_one(account: &Account, poll: &Poll<'_>, burn: &mut BurnHistory, now_ms: i64) -> PollResult {
    let dir = Path::new(&account.cli_dir);
    let live = fetch(dir, poll);
    let logged_in = poll.creds.access_token(dir).is_ok();
    let expires_at = poll.creds.expires_at(dir);
    let (usage, failure) = match live {
        Ok(mut usage) => {
            usage.read_at = Some(iso_from_ms(now_ms));
            if let Some(limit) = binding_limit(&usage) {
                burn.record(&account.key, now_ms, limit.percent);
            }
            (usage, None)
        }
        Err(e) => (fallback(account, poll.app, &e), Some(e)),
    };
    PollResult {
        key: account.key.clone(),
        usage,
        burn_per_min: burn.per_min(&account.key),
        expires_at,
        logged_in,
        failure,
    }
}

fn fetch(dir: &Path, poll: &Poll<'_>) -> Result<Usage, String> {
    let token = poll.creds.access_token(dir)?;
    match poll.source.fetch(&token) {
        Err(e) if e.starts_with(UNAUTHORIZED) => {
            let fresh = poll.creds.refresh(dir)?;
            poll.source.fetch(&fresh)
        }
        other => other,
    }
}

fn fallback(account: &Account, app: Option<&Path>, failure: &str) -> Usage {
    let logged = app
        .zip(account.desktop_org.as_deref())
        .and_then(|(app, org)| latest_sample(app, org));
    let mut usage = logged.unwrap_or_default();
    usage.error = Some(if usage.limits.is_empty() {
        failure.to_owned()
    } else {
        FALLBACK_ERROR.to_owned()
    });
    usage
}

/// How long to wait before the next pass: the fast interval once any binding limit is at
/// `fast_at` or more, else the normal one.
#[must_use]
pub fn next_interval(config: &AccountsConfig, results: &[PollResult]) -> Duration {
    let hot = results
        .iter()
        .filter_map(|r| binding_limit(&r.usage))
        .any(|l| l.percent >= config.fast_at);
    if hot {
        Duration::from_secs(config.fast_poll_seconds.max(1))
    } else {
        Duration::from_secs(config.poll_minutes.max(1) * 60)
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::fixtures::account_in;
    use crate::types::Severity;

    struct FakeSource(Mutex<Vec<Result<Usage, String>>>);
    impl UsageSource for FakeSource {
        fn fetch(&self, _token: &str) -> Result<Usage, String> {
            self.0.lock().unwrap().remove(0)
        }
    }

    struct FakeCreds {
        refresh: Result<String, String>,
    }
    impl Credentials for FakeCreds {
        fn access_token(&self, _dir: &Path) -> Result<String, String> {
            Ok("old".to_owned())
        }
        fn expires_at(&self, _dir: &Path) -> Option<i64> {
            Some(5)
        }
        fn refresh(&self, _dir: &Path) -> Result<String, String> {
            self.refresh.clone()
        }
    }

    fn limit(percent: f64, active: bool) -> UsageLimit {
        UsageLimit {
            kind: "weekly_all".to_owned(),
            group: None,
            percent,
            severity: Severity::Normal,
            resets_at: None,
            model: None,
            is_active: active,
        }
    }

    fn usage(percent: f64) -> Usage {
        Usage {
            limits: vec![limit(percent, true)],
            ..Usage::default()
        }
    }

    fn account(org: Option<&str>) -> Account {
        account_in("a", "A", None, org.map(str::to_owned))
    }

    fn run(source: Vec<Result<Usage, String>>, refresh: Result<String, String>) -> PollResult {
        let source = FakeSource(Mutex::new(source));
        let creds = FakeCreds { refresh };
        let poll = Poll {
            source: &source,
            creds: &creds,
            app: None,
        };
        let mut burn = BurnHistory::default();
        poll_all(&[account(None)], &poll, &mut burn, 1000).remove(0)
    }

    #[test]
    fn a_401_refreshes_once_and_retries() {
        let r = run(
            vec![Err("unauthorized: no".to_owned()), Ok(usage(40.0))],
            Ok("new".to_owned()),
        );
        assert!(r.failure.is_none());
        assert!((r.usage.limits[0].percent - 40.0).abs() < f64::EPSILON);
        assert_eq!(r.expires_at, Some(5));
        assert!(r.usage.read_at.is_some());
    }

    #[test]
    fn a_failed_refresh_leaves_the_error() {
        let r = run(
            vec![Err("unauthorized: no".to_owned())],
            Err("needs a fresh login".to_owned()),
        );
        assert_eq!(r.failure.as_deref(), Some("needs a fresh login"));
        assert_eq!(r.usage.error.as_deref(), Some("needs a fresh login"));
    }

    #[test]
    fn the_desktop_log_answers_when_the_live_read_fails() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("plan-usage-history.json"),
            r#"{"samples":[{"t":5,"org":"o","u":{"fh":7,"sd":8}}]}"#,
        )
        .unwrap();
        let u = fallback(&account(Some("o")), Some(dir.path()), "boom");
        assert_eq!(u.limits.len(), 2);
        assert_eq!(u.error.as_deref(), Some(FALLBACK_ERROR));
    }

    #[test]
    fn burn_is_the_slope_over_the_window_and_resets_on_a_drop() {
        let mut burn = BurnHistory::default();
        burn.record("a", 0, 10.0);
        assert!(burn.per_min("a").is_none());
        burn.record("a", 5 * 60_000, 20.0);
        assert!((burn.per_min("a").unwrap() - 2.0).abs() < 1e-9);
        burn.record("a", 6 * 60_000, 3.0);
        assert!(burn.per_min("a").is_none());
        burn.record("a", 40 * 60_000, 4.0);
        assert!(burn.per_min("a").is_none());
    }

    #[test]
    fn the_binding_limit_prefers_active_ones() {
        let u = Usage {
            limits: vec![limit(90.0, false), limit(40.0, true), limit(60.0, true)],
            ..Usage::default()
        };
        assert!((binding_limit(&u).unwrap().percent - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn the_interval_speeds_up_near_the_limit() {
        let config = AccountsConfig::default();
        let result = |p| PollResult {
            key: "a".to_owned(),
            usage: usage(p),
            burn_per_min: None,
            expires_at: None,
            logged_in: true,
            failure: None,
        };
        assert_eq!(
            next_interval(&config, &[result(10.0)]),
            Duration::from_secs(config.poll_minutes * 60)
        );
        assert_eq!(
            next_interval(&config, &[result(10.0), result(config.fast_at)]),
            Duration::from_secs(config.fast_poll_seconds)
        );
    }
}
