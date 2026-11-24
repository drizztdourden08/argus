//! The next switch of Desktop: the plan the owner may set (which account Desktop leaves and
//! which it goes to), the target the auto-switch takes, and the `next` the Accounts tab shows.

use serde::{Deserialize, Serialize};

use crate::rules::{View, pick_target};
use crate::types::{Account, AccountKey};

/// The switch the owner planned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextPlan {
    /// The account Desktop leaves.
    pub from: AccountKey,
    /// The account Desktop goes to.
    pub to: AccountKey,
}

/// Where the shown switch comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    /// The owner planned it.
    Owner,
    /// The auto-switch would pick it now.
    Auto,
}

/// When the switch is expected.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum When {
    /// At this percent of the leaving account's binding limit.
    Percent {
        /// The switch point.
        percent: f64,
    },
    /// At this time, projected from the burn rate.
    Time {
        /// Epoch milliseconds.
        #[serde(rename = "atMs")]
        at_ms: i64,
    },
    /// Nothing is known yet.
    Unknown,
}

/// The `next` of `GET /api/accounts`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Next {
    /// The account Desktop leaves.
    pub from: Option<AccountKey>,
    /// The account Desktop goes to, when one can take it.
    pub to: Option<AccountKey>,
    /// Who chose it.
    pub source: Source,
    /// When it is expected.
    pub when: When,
}

/// A plan from two keys, or the plain sentence that says why not.
///
/// # Errors
/// When the keys are equal, one is unknown, or `to` cannot receive.
pub fn check_plan(
    accounts: &[Account],
    views: &[View],
    from: &str,
    to: &str,
) -> Result<NextPlan, String> {
    if from == to {
        return Err("Pick two different accounts.".to_owned());
    }
    for key in [from, to] {
        if !accounts.iter().any(|a| a.key == key) {
            return Err(format!("{key} is not a known account."));
        }
    }
    if !views.iter().any(|v| v.key == to && v.can_receive) {
        return Err(format!(
            "{to} cannot receive a switch: it is turned off for receiving or has no saved Desktop login."
        ));
    }
    Ok(NextPlan {
        from: from.to_owned(),
        to: to.to_owned(),
    })
}

/// The account the auto-switch goes to from `from`: the planned one when the plan leaves from
/// `from` and it has room, else the one with the most room.
#[must_use]
pub fn target<'a>(accounts: &'a [View], from: &str, plan: Option<&NextPlan>) -> Option<&'a View> {
    let planned = plan
        .filter(|p| p.from == from)
        .and_then(|p| accounts.iter().find(|v| v.key == p.to))
        .filter(|v| v.key != from && v.has_room());
    planned.or_else(|| pick_target(accounts, from))
}

/// When the account is expected to reach its switch point: a time when the burn rate projects
/// one before the limit resets, else the switch point itself.
#[must_use]
pub fn when_of(view: Option<&View>, burn_per_min: Option<f64>, now_ms: i64) -> When {
    let Some((view, percent)) = view.and_then(|v| Some((v, v.percent?))) else {
        return When::Unknown;
    };
    let at = view.rules.switch_at;
    if view.over() {
        return When::Time { at_ms: now_ms };
    }
    let projected = burn_per_min.filter(|b| *b > 0.0).map(|burn| {
        let minutes = (at - percent) / burn;
        // Truncation to whole milliseconds is the point.
        #[allow(clippy::cast_possible_truncation)]
        let ms = (minutes * 60_000.0) as i64;
        now_ms.saturating_add(ms)
    });
    match projected {
        Some(ms) if view.resets_at_ms.is_none_or(|reset| ms < reset) => When::Time { at_ms: ms },
        _ => When::Percent { percent: at },
    }
}

/// What to show as the next switch.
#[must_use]
pub fn next_of(
    views: &[View],
    active: Option<&str>,
    plan: Option<&NextPlan>,
    burn: &dyn Fn(&str) -> Option<f64>,
    now_ms: i64,
) -> Next {
    let (from, to, source) = plan.map_or_else(
        || {
            let to = active
                .and_then(|a| pick_target(views, a))
                .map(|v| v.key.clone());
            (active.map(str::to_owned), to, Source::Auto)
        },
        |p| (Some(p.from.clone()), Some(p.to.clone()), Source::Owner),
    );
    let leaving = from
        .as_deref()
        .and_then(|key| views.iter().find(|v| v.key == key));
    let burn = from.as_deref().and_then(burn);
    Next {
        when: when_of(leaving, burn, now_ms),
        from,
        to,
        source,
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{HandoverMode, Rules};

    fn view(key: &str, percent: f64, can_receive: bool) -> View {
        View {
            key: key.to_owned(),
            label: key.to_uppercase(),
            can_receive,
            rules: Rules {
                auto_switch: true,
                switch_at: 98.0,
                handover: HandoverMode::Off,
                can_receive,
            },
            percent: Some(percent),
            critical: false,
            resets_at_ms: Some(10_000_000),
        }
    }

    fn account(key: &str) -> Account {
        serde_json::from_value(serde_json::json!({
            "key": key, "label": key, "color": "#000000", "cliDir": "x"
        }))
        .unwrap()
    }

    fn plan(from: &str, to: &str) -> NextPlan {
        NextPlan {
            from: from.to_owned(),
            to: to.to_owned(),
        }
    }

    fn key_of(view: Option<&View>) -> Option<&str> {
        view.map(|v| v.key.as_str())
    }

    #[test]
    fn a_plan_is_refused_with_a_sentence() {
        let accounts = [account("a"), account("b")];
        let views = [view("a", 10.0, true), view("b", 10.0, false)];
        let refuse = |from, to| check_plan(&accounts, &views, from, to).unwrap_err();
        assert_eq!(refuse("a", "a"), "Pick two different accounts.");
        assert_eq!(refuse("a", "z"), "z is not a known account.");
        assert!(refuse("a", "b").starts_with("b cannot receive"));
        assert_eq!(check_plan(&accounts, &views, "b", "a"), Ok(plan("b", "a")));
    }

    #[test]
    fn the_planned_target_wins_when_it_has_room() {
        let views = [
            view("a", 99.0, true),
            view("b", 50.0, true),
            view("c", 5.0, true),
        ];
        let chosen = target(&views, "a", Some(&plan("a", "b")));
        assert_eq!(key_of(chosen), Some("b"));
    }

    #[test]
    fn a_full_or_waiting_plan_falls_back_to_the_most_room() {
        let views = [
            view("a", 99.0, true),
            view("b", 99.0, true),
            view("c", 5.0, true),
        ];
        let full = target(&views, "a", Some(&plan("a", "b")));
        assert_eq!(key_of(full), Some("c"));
        let waits = target(&views, "a", Some(&plan("c", "b")));
        assert_eq!(key_of(waits), Some("c"));
    }

    #[test]
    fn the_time_comes_from_the_burn_and_else_the_switch_point() {
        let a = view("a", 90.0, true);
        assert_eq!(
            when_of(Some(&a), Some(1.0), 0),
            When::Time { at_ms: 480_000 }
        );
        assert_eq!(when_of(Some(&a), None, 0), When::Percent { percent: 98.0 });
        let slow = when_of(Some(&a), Some(0.000_000_1), 0);
        assert_eq!(slow, When::Percent { percent: 98.0 });
        assert_eq!(when_of(None, Some(1.0), 0), When::Unknown);
        let spent = view("a", 100.0, true);
        assert_eq!(when_of(Some(&spent), None, 7), When::Time { at_ms: 7 });
    }

    #[test]
    fn with_no_plan_the_auto_pick_is_shown() {
        let views = [view("a", 90.0, true), view("b", 20.0, true)];
        let shown = next_of(&views, Some("a"), None, &|_| None, 0);
        assert_eq!(shown.source, Source::Auto);
        assert_eq!(shown.from.as_deref(), Some("a"));
        assert_eq!(shown.to.as_deref(), Some("b"));
        let json = serde_json::to_value(&shown).unwrap();
        let want = serde_json::json!({ "kind": "percent", "percent": 98.0 });
        assert_eq!(json["when"], want);
        let owned = next_of(&views, Some("b"), Some(&plan("a", "b")), &|_| None, 0);
        assert_eq!(owned.source, Source::Owner);
        assert_eq!(owned.from.as_deref(), Some("a"));
    }
}
