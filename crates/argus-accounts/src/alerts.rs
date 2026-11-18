//! What to tell the owner about usage: a warning near a limit, a proposal to switch account and
//! the news that a spent limit is back. Pure: the caller gives the last and the new usage of
//! every account and sends the alerts. Each alert fires once per limit window; what fired is
//! remembered by account, limit and the hour the limit resets. The reset time the usage feed
//! gives drifts by seconds from one poll to the next, so the exact time would fire again.
//!
//! An account with auto-switch gets no proposal: the switch rules count down for it.

use std::collections::HashSet;

use argus_core::clock::parse_ms;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;
use time::{OffsetDateTime, UtcOffset};

use crate::types::{Account, AccountKey, Severity, Usage, UsageLimit};

/// A limit at or above this is spent; the reset alert starts from it.
const SPENT: f64 = 99.0;
/// A spent limit that falls under this has reset.
const RESET_BELOW: f64 = 50.0;
const WHEN: &[BorrowedFormatItem<'static>] = format_description!(
    "[weekday repr:short] [month repr:short] [day padding:none] at [hour repr:12 padding:none]:[minute] [period]"
);

/// One account with its usage and whether Desktop runs under it now.
#[derive(Debug, Clone)]
pub struct Snapshot {
    /// The account and its rules.
    pub account: Account,
    /// Its usage.
    pub usage: Usage,
    /// Desktop is signed in as this account.
    pub active: bool,
}

/// The settings an evaluation needs.
#[derive(Debug, Clone, Copy)]
pub struct AlertSettings {
    /// The percent a warning starts at.
    pub warn_at: f64,
    /// The percent a switch is proposed at.
    pub propose_at: f64,
    /// The port of this server, for the address a toast opens.
    pub port: u16,
    /// Seconds east of UTC, for the times in the texts.
    pub offset_seconds: i32,
}

/// The kind of an alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertKind {
    /// A limit is near its end.
    Warn,
    /// A limit is nearly spent and another account may have room.
    ProposeSwitch,
    /// A spent limit is back.
    Reset,
}

/// One toast to send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alert {
    /// What it is.
    pub kind: AlertKind,
    /// The account it is about.
    pub account: AccountKey,
    /// The account a switch would go to, for a proposal.
    pub target: Option<AccountKey>,
    /// The title.
    pub title: String,
    /// The text.
    pub text: String,
    /// The address a click opens.
    pub url: String,
}

/// Remembers which alerts fired for which limit window.
#[derive(Debug, Default)]
pub struct Alerter {
    fired: HashSet<String>,
}

impl Alerter {
    /// An alerter that has fired nothing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The alerts the change from `previous` to `current` calls for, at `now_ms`.
    pub fn check(
        &mut self,
        previous: &[Snapshot],
        current: &[Snapshot],
        settings: &AlertSettings,
        now_ms: i64,
    ) -> Vec<Alert> {
        let mut alerts = Vec::new();
        for now in current {
            for limit in &now.usage.limits {
                alerts.extend(self.rising(now, limit, current, settings));
            }
            if let Some(before) = previous.iter().find(|p| p.account.key == now.account.key) {
                self.resets(before, now, settings, now_ms, &mut alerts);
            }
        }
        alerts
    }

    /// True the first time this alert is asked about for the limit's window.
    fn once(&mut self, account: &str, what: &str, limit: &UsageLimit) -> bool {
        let id = format!("{account}|{what}|{}|{:?}", limit_id(limit), window(limit));
        self.fired.insert(id)
    }

    fn rising(
        &mut self,
        now: &Snapshot,
        limit: &UsageLimit,
        all: &[Snapshot],
        settings: &AlertSettings,
    ) -> Option<Alert> {
        let key = &now.account.key;
        if !counts(limit) || limit.percent < settings.warn_at {
            return None;
        }
        let propose = now.active
            && !now.account.rules.auto_switch
            && (limit.percent >= settings.propose_at || limit.severity == Severity::Critical);
        if propose {
            let fresh = self.once(key, "propose", limit);
            self.once(key, "warn", limit);
            return fresh.then(|| propose_alert(now, limit, best_target(now, all), settings));
        }
        self.once(key, "warn", limit)
            .then(|| warn_alert(now, limit, settings))
    }

    fn resets(
        &mut self,
        before: &Snapshot,
        now: &Snapshot,
        settings: &AlertSettings,
        now_ms: i64,
        alerts: &mut Vec<Alert>,
    ) {
        for old in before.usage.limits.iter().filter(|l| l.percent >= SPENT) {
            let Some(new) = now
                .usage
                .limits
                .iter()
                .find(|l| limit_id(l) == limit_id(old))
            else {
                continue;
            };
            let passed = ms_of(old.resets_at.as_deref()).is_some_and(|t| t <= now_ms);
            let back = new.percent < RESET_BELOW || (passed && new.percent < SPENT);
            if back && self.once(&now.account.key, "reset", old) {
                alerts.push(reset_alert(now, new, settings));
            }
        }
    }
}

/// Seconds east of UTC of this PC's local time zone, 0 when it cannot be told.
#[must_use]
pub fn local_offset_seconds() -> i32 {
    UtcOffset::current_local_offset().map_or(0, UtcOffset::whole_seconds)
}

fn counts(limit: &UsageLimit) -> bool {
    limit.is_active || limit.severity != Severity::Normal
}

fn limit_id(limit: &UsageLimit) -> String {
    format!(
        "{}/{}/{}",
        limit.kind,
        limit.group.as_deref().unwrap_or_default(),
        limit.model.as_deref().unwrap_or_default()
    )
}

/// The hour a limit resets, rounded to the nearest one, so a drift of seconds is the same window.
fn window(limit: &UsageLimit) -> Option<i64> {
    const HOUR_MS: i64 = 3_600_000;
    ms_of(limit.resets_at.as_deref()).map(|t| (t + HOUR_MS / 2).div_euclid(HOUR_MS))
}

fn ms_of(text: Option<&str>) -> Option<i64> {
    text.and_then(parse_ms)
}

/// The busiest percent of an account: its active limits, else all of them.
fn busy_percent(usage: &Usage) -> Option<f64> {
    let top = |active_only: bool| {
        usage
            .limits
            .iter()
            .filter(|l| l.is_active || !active_only)
            .map(|l| l.percent)
            .reduce(f64::max)
    };
    top(true).or_else(|| top(false))
}

fn soonest_reset(usage: &Usage) -> i64 {
    usage
        .limits
        .iter()
        .filter_map(|l| ms_of(l.resets_at.as_deref()))
        .min()
        .unwrap_or(i64::MAX)
}

/// The other account a switch should go to: it can receive, is under its own switch percent,
/// has the lowest use, then the soonest reset.
fn best_target<'a>(from: &Snapshot, all: &'a [Snapshot]) -> Option<&'a Snapshot> {
    all.iter()
        .filter(|s| s.account.key != from.account.key && s.account.can_receive)
        .filter_map(|s| busy_percent(&s.usage).map(|p| (s, p)))
        .filter(|(s, p)| *p < s.account.rules.switch_at)
        .min_by(|(a, pa), (b, pb)| {
            pa.total_cmp(pb)
                .then_with(|| soonest_reset(&a.usage).cmp(&soonest_reset(&b.usage)))
        })
        .map(|(s, _)| s)
}

fn limit_name(limit: &UsageLimit) -> String {
    match (limit.kind.as_str(), limit.model.as_deref()) {
        ("weekly_all", _) => "weekly limit".to_owned(),
        ("weekly_scoped", Some(model)) => format!("weekly {model} limit"),
        ("five_hour", _) => "five hour limit".to_owned(),
        (kind, _) => format!("{} limit", kind.replace('_', " ")),
    }
}

fn clock_text(resets_at: Option<&str>, settings: &AlertSettings) -> Option<String> {
    let ms = ms_of(resets_at)?;
    let at = OffsetDateTime::from_unix_timestamp_nanos(i128::from(ms) * 1_000_000).ok()?;
    let offset = UtcOffset::from_whole_seconds(settings.offset_seconds).ok()?;
    at.to_offset(offset).format(WHEN).ok()
}

fn used_and_reset(limit: &UsageLimit, settings: &AlertSettings) -> String {
    let used = format!("{:.0}% used", limit.percent);
    match clock_text(limit.resets_at.as_deref(), settings) {
        Some(when) => format!("{used}, resets {when}."),
        None => format!("{used}."),
    }
}

fn address(settings: &AlertSettings, target: Option<&str>) -> String {
    let base = format!("http://127.0.0.1:{}/?tab=accounts", settings.port);
    match target {
        Some(key) => format!("{base}&switch={key}"),
        None => base,
    }
}

fn warn_alert(now: &Snapshot, limit: &UsageLimit, settings: &AlertSettings) -> Alert {
    Alert {
        kind: AlertKind::Warn,
        account: now.account.key.clone(),
        target: None,
        title: format!("{} is near its {}", now.account.label, limit_name(limit)),
        text: used_and_reset(limit, settings),
        url: address(settings, None),
    }
}

fn propose_alert(
    now: &Snapshot,
    limit: &UsageLimit,
    target: Option<&Snapshot>,
    settings: &AlertSettings,
) -> Alert {
    let label = &now.account.label;
    let title = if limit.percent >= SPENT {
        format!("{label} hit its {}", limit_name(limit))
    } else {
        format!("{label} is nearly at its {}", limit_name(limit))
    };
    let advice = match target {
        Some(t) => format!(
            "{} is at {:.0}%.",
            t.account.label,
            busy_percent(&t.usage).unwrap_or_default()
        ),
        None => "No other account has room.".to_owned(),
    };
    Alert {
        kind: AlertKind::ProposeSwitch,
        account: now.account.key.clone(),
        target: target.map(|t| t.account.key.clone()),
        title,
        text: format!("{} {advice}", used_and_reset(limit, settings)),
        url: address(settings, target.map(|t| t.account.key.as_str())),
    }
}

fn reset_alert(now: &Snapshot, limit: &UsageLimit, settings: &AlertSettings) -> Alert {
    Alert {
        kind: AlertKind::Reset,
        account: now.account.key.clone(),
        target: None,
        title: format!("{} has reset", now.account.label),
        text: format!(
            "Its {} is back to {:.0}% used.",
            limit_name(limit),
            limit.percent
        ),
        url: address(settings, None),
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::{Alert, AlertKind, AlertSettings, Alerter, Snapshot};
    use crate::types::{Account, Rules, Severity, Usage, UsageLimit};

    const NOW: i64 = 1_763_000_000_000;
    const FUTURE: &str = "2026-11-14T08:00:00Z";

    fn settings() -> AlertSettings {
        AlertSettings {
            warn_at: 80.0,
            propose_at: 95.0,
            port: 4000,
            offset_seconds: -5 * 3600,
        }
    }

    fn limit(percent: f64, resets_at: &str) -> UsageLimit {
        UsageLimit {
            kind: "weekly_all".to_owned(),
            group: None,
            percent,
            severity: Severity::Normal,
            resets_at: Some(resets_at.to_owned()),
            model: None,
            is_active: true,
        }
    }

    fn proposal(alerts: &[Alert]) -> &Alert {
        alerts
            .iter()
            .find(|a| a.kind == AlertKind::ProposeSwitch)
            .unwrap()
    }

    fn snap(key: &str, percent: f64, active: bool, auto: bool) -> Snapshot {
        Snapshot {
            account: Account {
                key: key.to_owned(),
                label: key.to_uppercase(),
                color: "#000000".to_owned(),
                desktop_account: None,
                desktop_org: None,
                cli_dir: String::new(),
                email: None,
                plan: None,
                can_receive: true,
                rules: Rules {
                    auto_switch: auto,
                    ..Rules::default()
                },
            },
            usage: Usage {
                limits: vec![limit(percent, FUTURE)],
                ..Usage::default()
            },
            active,
        }
    }

    #[test]
    fn warns_once_per_window() {
        let mut alerter = Alerter::new();
        let all = [snap("a", 85.0, false, false)];
        let first = alerter.check(&[], &all, &settings(), NOW);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].kind, AlertKind::Warn);
        assert!(alerter.check(&all, &all, &settings(), NOW).is_empty());
        let next = [Snapshot {
            usage: Usage {
                limits: vec![limit(90.0, "2026-11-21T08:00:00Z")],
                ..Usage::default()
            },
            ..all[0].clone()
        }];
        assert_eq!(alerter.check(&all, &next, &settings(), NOW).len(), 1);
    }

    #[test]
    fn a_reset_time_that_drifts_by_seconds_is_the_same_window() {
        let mut alerter = Alerter::new();
        let at = |percent: f64, resets: &str| {
            let mut s = snap("a", percent, true, false);
            s.usage.limits = vec![limit(percent, resets)];
            [s]
        };
        let first = at(92.0, "2026-11-21T07:59:58.120Z");
        assert_eq!(alerter.check(&[], &first, &settings(), NOW).len(), 1);
        let drift = at(92.0, "2026-11-21T08:00:00.480Z");
        assert_eq!(alerter.check(&first, &drift, &settings(), NOW).len(), 0);
        let earlier = at(93.0, "2026-11-21T07:59:31Z");
        assert_eq!(alerter.check(&drift, &earlier, &settings(), NOW).len(), 0);
    }

    #[test]
    fn proposes_the_emptiest_other_account() {
        let mut alerter = Alerter::new();
        let all = [
            snap("second", 100.0, true, false),
            snap("hotmail", 2.0, false, false),
            snap("work", 40.0, false, false),
        ];
        let alerts = alerter.check(&[], &all, &settings(), NOW);
        let propose = proposal(&alerts);
        assert_eq!(propose.target.as_deref(), Some("hotmail"));
        assert_eq!(
            propose.url,
            "http://127.0.0.1:4000/?tab=accounts&switch=hotmail"
        );
        assert_eq!(propose.title, "SECOND hit its weekly limit");
        assert_eq!(
            propose.text,
            "100% used, resets Sat Nov 14 at 3:00 AM. HOTMAIL is at 2%."
        );
        assert!(alerts.iter().all(|a| a.kind != AlertKind::Warn));
    }

    #[test]
    fn no_proposal_with_auto_switch_or_when_inactive() {
        let mut alerter = Alerter::new();
        let auto = [snap("a", 99.0, true, true), snap("b", 1.0, false, false)];
        let kinds: Vec<_> = alerter
            .check(&[], &auto, &settings(), NOW)
            .into_iter()
            .map(|a| a.kind)
            .collect();
        assert_eq!(kinds, [AlertKind::Warn]);
        let idle = [snap("c", 99.0, false, false)];
        assert!(
            alerter
                .check(&[], &idle, &settings(), NOW)
                .iter()
                .all(|a| a.kind == AlertKind::Warn)
        );
    }

    #[test]
    fn a_target_must_be_under_its_own_switch_percent_and_receive() {
        let mut alerter = Alerter::new();
        let mut full = snap("b", 99.0, false, false);
        full.account.rules.switch_at = 98.0;
        let mut closed = snap("c", 1.0, false, false);
        closed.account.can_receive = false;
        let all = [snap("a", 100.0, true, false), full, closed];
        let alerts = alerter.check(&[], &all, &settings(), NOW);
        let propose = proposal(&alerts);
        assert_eq!(propose.target, None);
        assert!(propose.text.ends_with("No other account has room."));
    }

    #[test]
    fn a_spent_limit_that_drops_has_reset() {
        let mut alerter = Alerter::new();
        let spent = [snap("a", 100.0, false, false)];
        let low = [snap("a", 3.0, false, false)];
        let alerts = alerter.check(&spent, &low, &settings(), NOW);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].kind, AlertKind::Reset);
        assert_eq!(alerts[0].title, "A has reset");
        assert!(alerter.check(&spent, &low, &settings(), NOW).is_empty());
    }

    #[test]
    fn a_passed_reset_time_counts_as_reset() {
        let mut alerter = Alerter::new();
        let spent = [snap("a", 100.0, false, false)];
        let later = 1_900_000_000_000;
        let alerts = alerter.check(&spent, &[snap("a", 60.0, false, false)], &settings(), later);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].kind, AlertKind::Reset);
    }
}
