//! One poll pass over all accounts, the burn rate and the poll interval.

use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::time::Duration;

use argus_core::clock::iso_from_ms;

use super::fallback::latest_sample;
use super::{UNAUTHORIZED, rate_limit_of};
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
    /// When the CLI login expires (ISO), as `claude auth status` last said; filled in by the
    /// caller, never by the poll.
    pub login_expires_at: Option<String>,
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
    /// The settings: the intervals and `fast_at`.
    pub config: &'a AccountsConfig,
    /// The Desktop account Claude Desktop is on now, if it is known.
    pub active: Option<&'a str>,
}

/// No account is read more often than this, whatever the settings say.
pub const MIN_GAP: Duration = Duration::from_secs(60);
/// The longest an account is backed off after a 429.
pub const MAX_BACKOFF: Duration = Duration::from_secs(30 * 60);
/// The error shown on the last good read of a rate-limited account, before the time.
const RATE_LIMITED_TEXT: &str = "rate limited, retrying at ";

/// When each account may be read next, and what it showed last.
#[derive(Debug, Default)]
pub struct Schedule {
    slots: HashMap<String, Slot>,
}

#[derive(Debug, Default)]
struct Slot {
    /// Not read again before this (epoch milliseconds).
    not_before: i64,
    /// The wait of the last 429, doubled by the next one; zero after a success.
    backoff_ms: i64,
    /// The last result of the account, shown again while it is not read.
    last: Option<PollResult>,
    /// The last good read.
    good: Option<Usage>,
}

impl Schedule {
    /// How long until the first account is due, at least a second.
    #[must_use]
    pub fn wait(&self, now_ms: i64) -> Duration {
        let soonest = self.slots.values().map(|s| s.not_before).min();
        let ms = soonest.map_or(millis(MIN_GAP), |at| at - now_ms);
        Duration::from_millis(u64::try_from(ms.max(1000)).unwrap_or(1000))
    }
}

fn millis(d: Duration) -> i64 {
    i64::try_from(d.as_millis()).unwrap_or(i64::MAX)
}

/// Reads the accounts that are due, once. An account that is not due (it was read less than a
/// minute ago, or it is backed off after a 429) keeps its last result.
pub fn poll_all(
    accounts: &[Account],
    poll: &Poll<'_>,
    state: (&mut BurnHistory, &mut Schedule),
    now_ms: i64,
) -> Vec<PollResult> {
    let (burn, schedule) = state;
    schedule
        .slots
        .retain(|key, _| accounts.iter().any(|a| &a.key == key));
    accounts
        .iter()
        .map(|account| {
            let slot = schedule.slots.entry(account.key.clone()).or_default();
            if let Some(last) = slot.last.as_ref().filter(|_| now_ms < slot.not_before) {
                return last.clone();
            }
            let result = poll_one(account, poll, burn, slot, now_ms);
            slot.last = Some(result.clone());
            result
        })
        .collect()
}

fn poll_one(
    account: &Account,
    poll: &Poll<'_>,
    burn: &mut BurnHistory,
    slot: &mut Slot,
    now_ms: i64,
) -> PollResult {
    let dir = Path::new(&account.cli_dir);
    let live = fetch(dir, poll);
    let logged_in = poll.creds.access_token(dir).is_ok();
    let (usage, failure) = match live {
        Ok(mut usage) => {
            usage.read_at = Some(iso_from_ms(now_ms));
            if let Some(limit) = binding_limit(&usage) {
                burn.record(&account.key, now_ms, limit.percent);
            }
            let wait = next_interval(poll.config, account, poll.active, &usage);
            slot.not_before = now_ms + millis(wait);
            slot.backoff_ms = 0;
            slot.good = Some(usage.clone());
            (usage, None)
        }
        Err(e) => {
            if let Some(asked) = rate_limit_of(&e) {
                rate_limited_read(account, poll, slot, asked, now_ms)
            } else {
                slot.not_before = now_ms + millis(MIN_GAP);
                (fallback(account, poll.app, &e), Some(e))
            }
        }
    };
    PollResult {
        key: account.key.clone(),
        usage,
        burn_per_min: burn.per_min(&account.key),
        login_expires_at: None,
        logged_in,
        failure,
    }
}

/// The result of a 429: the last good read with the time of the next try. The account waits what
/// `Retry-After` asked for, else twice its last wait (its own interval the first time), at most
/// [`MAX_BACKOFF`] and at least [`MIN_GAP`].
fn rate_limited_read(
    account: &Account,
    poll: &Poll<'_>,
    slot: &mut Slot,
    asked: Option<u64>,
    now_ms: i64,
) -> (Usage, Option<String>) {
    let own = slot.good.as_ref().map_or_else(
        || next_interval(poll.config, account, poll.active, &Usage::default()),
        |u| next_interval(poll.config, account, poll.active, u),
    );
    let wait = match asked {
        Some(secs) => Duration::from_secs(secs),
        None if slot.backoff_ms > 0 => Duration::from_millis(slot.backoff_ms.unsigned_abs() * 2),
        None => own * 2,
    };
    let wait = wait.clamp(MIN_GAP, MAX_BACKOFF);
    slot.backoff_ms = millis(wait);
    slot.not_before = now_ms + millis(wait);
    let text = format!("{RATE_LIMITED_TEXT}{}", iso_from_ms(slot.not_before));
    let mut usage = slot
        .good
        .clone()
        .unwrap_or_else(|| fallback(account, poll.app, &text));
    usage.error = Some(text.clone());
    (usage, Some(text))
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

/// How long to wait before reading `account` again. The fast interval applies only to the account
/// Desktop is on, and only while its binding limit is from `fast_at` up to 99 percent: an account
/// at its limit waits for the reset at the normal interval. Never under a minute.
#[must_use]
pub fn next_interval(
    config: &AccountsConfig,
    account: &Account,
    active: Option<&str>,
    usage: &Usage,
) -> Duration {
    let on_desktop = active.is_some() && account.desktop_account.as_deref() == active;
    let hot =
        binding_limit(usage).is_some_and(|l| l.percent >= config.fast_at && l.percent <= 99.0);
    let wait = if on_desktop && hot {
        Duration::from_secs(config.fast_poll_seconds.max(1))
    } else {
        Duration::from_secs(config.poll_minutes.max(1) * 60)
    };
    wait.max(MIN_GAP)
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::fixtures::account_in;
    use crate::types::Severity;
    use crate::usage::rate_limited;

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

    fn polled(
        source: Vec<Result<Usage, String>>,
        refresh: Result<String, String>,
        schedule: &mut Schedule,
        now: i64,
    ) -> PollResult {
        let source = FakeSource(Mutex::new(source));
        let creds = FakeCreds { refresh };
        let config = AccountsConfig::default();
        let poll = Poll {
            source: &source,
            creds: &creds,
            app: None,
            config: &config,
            active: None,
        };
        let mut burn = BurnHistory::default();
        poll_all(&[account(None)], &poll, (&mut burn, schedule), now).remove(0)
    }

    fn run(source: Vec<Result<Usage, String>>, refresh: Result<String, String>) -> PollResult {
        polled(source, refresh, &mut Schedule::default(), 1000)
    }

    #[test]
    fn a_401_refreshes_once_and_retries() {
        let r = run(
            vec![Err("unauthorized: no".to_owned()), Ok(usage(40.0))],
            Ok("new".to_owned()),
        );
        assert!(r.failure.is_none());
        assert!((r.usage.limits[0].percent - 40.0).abs() < f64::EPSILON);
        assert_eq!(r.login_expires_at, None);
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

    fn on_desktop() -> Account {
        account_in("a", "A", Some("acc-1".to_owned()), None)
    }

    #[test]
    fn fast_polling_is_for_the_desktop_account_between_fast_at_and_99() {
        let config = AccountsConfig::default();
        let normal = Duration::from_secs(config.poll_minutes * 60);
        let fast = Duration::from_secs(config.fast_poll_seconds);
        let at = |account: &Account, active, p| next_interval(&config, account, active, &usage(p));
        assert_eq!(at(&on_desktop(), Some("acc-1"), 10.0), normal);
        assert_eq!(at(&on_desktop(), Some("acc-1"), config.fast_at), fast);
        assert_eq!(at(&on_desktop(), Some("acc-1"), 99.0), fast);
        assert_eq!(at(&on_desktop(), Some("acc-1"), 100.0), normal);
        assert_eq!(at(&on_desktop(), Some("other"), 90.0), normal);
        assert_eq!(at(&on_desktop(), None, 90.0), normal);
    }

    #[test]
    fn an_interval_is_never_under_a_minute() {
        let config = AccountsConfig {
            fast_poll_seconds: 5,
            ..AccountsConfig::default()
        };
        let wait = next_interval(&config, &on_desktop(), Some("acc-1"), &usage(90.0));
        assert_eq!(wait, MIN_GAP);
    }

    fn pass(source: Vec<Result<Usage, String>>, schedule: &mut Schedule, now: i64) -> PollResult {
        polled(source, Err("no".to_owned()), schedule, now)
    }

    #[test]
    fn a_429_backs_off_and_keeps_the_last_good_read() {
        let mut schedule = Schedule::default();
        let good = pass(vec![Ok(usage(40.0))], &mut schedule, 0);
        assert!(good.failure.is_none());
        let first = pass(vec![Err(rate_limited(None))], &mut schedule, 300_000);
        assert!((first.usage.limits[0].percent - 40.0).abs() < f64::EPSILON);
        let error = first.usage.error.clone().unwrap();
        assert!(error.starts_with("rate limited, retrying at "), "{error}");
        // The account's own interval is 5 minutes; the first back-off doubles it.
        assert_eq!(schedule.slots["a"].backoff_ms, 600_000);
        let second = pass(vec![Err(rate_limited(None))], &mut schedule, 900_000);
        assert!(second.usage.error.is_some());
        assert_eq!(schedule.slots["a"].backoff_ms, 1_200_000);
        let third = pass(vec![Err(rate_limited(None))], &mut schedule, 2_100_000);
        assert!(third.failure.is_some());
        assert_eq!(schedule.slots["a"].backoff_ms, 1_800_000);
        let fresh = pass(vec![Ok(usage(41.0))], &mut schedule, 4_000_000);
        assert_eq!(schedule.slots["a"].backoff_ms, 0);
        assert!(fresh.failure.is_none());
    }

    #[test]
    fn retry_after_is_honoured_and_floored_at_a_minute() {
        let mut schedule = Schedule::default();
        pass(vec![Err(rate_limited(Some(900)))], &mut schedule, 0);
        assert_eq!(schedule.slots["a"].not_before, 900_000);
        pass(vec![Err(rate_limited(Some(3)))], &mut schedule, 900_000);
        assert_eq!(schedule.slots["a"].not_before, 900_000 + 60_000);
    }

    #[test]
    fn an_account_is_not_read_again_inside_its_wait() {
        let mut schedule = Schedule::default();
        pass(vec![Ok(usage(10.0))], &mut schedule, 0);
        // The source would panic when asked: the second pass must not read.
        let again = pass(vec![], &mut schedule, 59_000);
        assert!((again.usage.limits[0].percent - 10.0).abs() < f64::EPSILON);
        assert_eq!(schedule.wait(59_000), Duration::from_secs(241));
    }

    #[test]
    fn the_429_error_round_trips() {
        assert_eq!(rate_limit_of(&rate_limited(Some(12))), Some(Some(12)));
        assert_eq!(rate_limit_of(&rate_limited(None)), Some(None));
        assert_eq!(rate_limit_of("boom"), None);
    }
}
