//! The switching rules of the accounts, as one pure function: what to do now, given the accounts
//! with their rules and usage, the account Claude runs as, the open handover cycle, the hold-offs
//! left by cancels, the countdown that runs and the clock. The server's rules service reads the
//! world into [`Facts`], calls [`decide`] and carries the [`Decision`]s out.
//!
//! ```text
//! switch point reached (switchAt, severity critical, or 100)   -> countdown, then switch
//! handover Off                                                  -> countdown straight away
//! handover At(n)    at n   -> cycle (fixed), ask every lead
//! handover Auto     near   -> cycle (auto) asks for estimates; at the ask point ask every lead
//! every asked lead ready (even below the switch point)          -> countdown at once
//! switch point with the cycle still open                        -> cut the rest off, countdown
//! countdown over: still over the point (or ready), target still has room, same account
//!                                                               -> switch, else stand down
//! ```

use crate::handover::CycleMode;
use crate::types::{AccountKey, HandoverMode, Rules};

/// The grace between the decision to switch and the switch.
pub const GRACE_MS: i64 = 60_000;
/// A cancel holds a rule off until the use is this many points under the switch point.
pub const HOLD_OFF_MARGIN: f64 = 5.0;

/// A lead, named by its board and thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeadKey {
    /// The board key.
    pub board: String,
    /// The lead's thread.
    pub thread: String,
}

/// One account as the rules see it.
#[derive(Debug, Clone, PartialEq)]
pub struct View {
    /// The account key.
    pub key: AccountKey,
    /// The name shown on screen.
    pub label: String,
    /// A switch may land here.
    pub can_receive: bool,
    /// Its switching rules.
    pub rules: Rules,
    /// Percent of its binding limit; `None` before a usage read.
    pub percent: Option<f64>,
    /// Whether the binding limit is flagged critical.
    pub critical: bool,
    /// When the binding limit resets, epoch milliseconds.
    pub resets_at_ms: Option<i64>,
}

impl View {
    /// Whether the use reached the switch point: its percent, a critical flag or 100.
    #[must_use]
    pub fn over(&self) -> bool {
        self.percent
            .is_some_and(|p| p >= self.rules.switch_at || p >= 100.0)
            || self.critical
    }

    /// Whether a switch may land here: it can receive and is under its own switch point.
    #[must_use]
    pub fn has_room(&self) -> bool {
        self.can_receive && self.percent.is_some_and(|p| p < self.rules.switch_at)
    }
}

/// Where the handover cycle of the active account stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleView {
    /// No cycle, or one that is over.
    None,
    /// A cycle is open for another account.
    Foreign,
    /// The leads were asked for estimates.
    Estimating,
    /// The leads were asked for their handover.
    Asked,
    /// Every asked lead is ready or cut off.
    Ready,
}

/// What the estimator says about the active account.
#[derive(Debug, Clone, PartialEq)]
pub struct Estimates {
    /// The use at which the leads are asked for their handover.
    pub ask_at: f64,
    /// Whether any lead is working.
    pub has_leads: bool,
    /// The leads to ask for an estimate now.
    pub ask_estimates: Vec<LeadKey>,
    /// Working leads the open cycle does not have yet.
    pub new_leads: Vec<LeadKey>,
}

/// A countdown that runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Countdown {
    /// The account being left.
    pub from: AccountKey,
    /// The account to switch to.
    pub to: AccountKey,
    /// When the grace ends, epoch milliseconds.
    pub ends_at_ms: i64,
}

/// A rule held off after a cancel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoldOff {
    /// The account whose rule is held off.
    pub account: AccountKey,
    /// The reset time of its binding limit when it was cancelled.
    pub resets_at_ms: Option<i64>,
}

impl HoldOff {
    /// The hold-off a cancel leaves on an account.
    #[must_use]
    pub fn after_cancel(view: &View) -> Self {
        HoldOff {
            account: view.key.clone(),
            resets_at_ms: view.resets_at_ms,
        }
    }
}

/// A limit window: one account and the reset time of its binding limit. A no-target notice is
/// sent once per window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    /// The account that ran out.
    pub account: AccountKey,
    /// When its binding limit resets.
    pub resets_at_ms: Option<i64>,
}

/// Everything [`decide`] reads.
pub struct Facts<'a> {
    /// Every account.
    pub accounts: &'a [View],
    /// The account Claude runs as now.
    pub active: Option<&'a str>,
    /// The estimator's word on the active account.
    pub estimates: Option<&'a Estimates>,
    /// The cycle of the active account.
    pub cycle: CycleView,
    /// The hold-offs.
    pub hold_offs: &'a [HoldOff],
    /// The countdown that runs.
    pub countdown: Option<&'a Countdown>,
    /// The window of the last no-target notice.
    pub told: Option<&'a Window>,
    /// Now, epoch milliseconds.
    pub now_ms: i64,
}

/// Why a countdown starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountdownReason {
    /// The switch point was reached.
    SwitchPoint,
    /// Every asked lead is ready.
    HandoversReady,
}

/// Why a countdown ends without a switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandDownReason {
    /// Claude runs as another account now.
    AccountChanged,
    /// The auto-switch rule was turned off.
    RuleOff,
    /// The use dropped below the switch point and no cycle is ready.
    BelowPoint,
    /// The target is over its own switch point or cannot receive any more.
    TargetFull,
}

impl StandDownReason {
    /// The reason in words for the owner.
    #[must_use]
    pub fn text(self) -> &'static str {
        match self {
            StandDownReason::AccountChanged => "Claude is on another account now",
            StandDownReason::RuleOff => "the auto-switch rule was turned off",
            StandDownReason::BelowPoint => "the use dropped below the switch point",
            StandDownReason::TargetFull => "the target account has no room left",
        }
    }
}

/// What the rules want done.
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    /// Ask these leads again for an estimate.
    StartEstimates(Vec<LeadKey>),
    /// Bring these leads into the open cycle.
    AddLeads(Vec<LeadKey>),
    /// Open a handover cycle for the active account.
    StartCycle {
        /// Fixed or auto.
        mode: CycleMode,
        /// The ask point of a fixed cycle.
        ask_at: Option<f64>,
    },
    /// Ask every lead of the cycle for its handover.
    AskAll,
    /// The switch point came first: cut the leads that are not ready off.
    CutOffRest,
    /// Start the one-minute grace.
    StartCountdown {
        /// The account to switch to.
        to: AccountKey,
        /// Why.
        reason: CountdownReason,
    },
    /// The grace is over and the checks hold: switch.
    Switch {
        /// The account to switch to.
        to: AccountKey,
    },
    /// The grace is over and a check failed.
    StandDown {
        /// Why.
        reason: StandDownReason,
    },
    /// Tell the owner no account can take over; once per window.
    NotifyNoTarget {
        /// The window told about.
        window: Window,
    },
    /// Forget the hold-off of an account.
    ClearHoldOff {
        /// The account.
        account: AccountKey,
    },
}

fn view_of<'a>(facts: &Facts<'a>, key: &str) -> Option<&'a View> {
    facts.accounts.iter().find(|v| v.key == key)
}

/// The account with the most room: lowest percent, then soonest reset, under its own switch
/// point and able to receive.
#[must_use]
pub fn pick_target<'a>(accounts: &'a [View], from: &str) -> Option<&'a View> {
    accounts
        .iter()
        .filter(|v| v.key != from && v.has_room())
        .min_by(|a, b| {
            let rank = |v: &View| (v.percent.unwrap_or(f64::MAX), v.resets_at_ms);
            let ((pa, ra), (pb, rb)) = (rank(a), rank(b));
            pa.total_cmp(&pb)
                .then_with(|| ra.unwrap_or(i64::MAX).cmp(&rb.unwrap_or(i64::MAX)))
        })
}

fn hold_off_over(hold: &HoldOff, view: Option<&View>, now_ms: i64) -> bool {
    let Some(view) = view else { return false };
    let reset =
        hold.resets_at_ms.is_some_and(|t| now_ms >= t) || hold.resets_at_ms != view.resets_at_ms;
    let dropped = view
        .percent
        .is_some_and(|p| p <= view.rules.switch_at - HOLD_OFF_MARGIN);
    !view.rules.auto_switch || reset || dropped
}

fn clearings(facts: &Facts<'_>) -> Vec<Decision> {
    facts
        .hold_offs
        .iter()
        .filter(|h| hold_off_over(h, view_of(facts, &h.account), facts.now_ms))
        .map(|h| Decision::ClearHoldOff {
            account: h.account.clone(),
        })
        .collect()
}

fn held(facts: &Facts<'_>, key: &str) -> bool {
    facts
        .hold_offs
        .iter()
        .any(|h| h.account == key && !hold_off_over(h, view_of(facts, key), facts.now_ms))
}

/// What to do now. Clearing hold-offs comes first, then the rules of the active account.
#[must_use]
pub fn decide(facts: &Facts<'_>) -> Vec<Decision> {
    let mut out = clearings(facts);
    out.extend(for_active(facts));
    out
}

fn for_active(facts: &Facts<'_>) -> Vec<Decision> {
    if let Some(countdown) = facts.countdown {
        return on_countdown(facts, countdown);
    }
    let Some(me) = facts.active.and_then(|key| view_of(facts, key)) else {
        return Vec::new();
    };
    if !me.rules.auto_switch || held(facts, &me.key) {
        return Vec::new();
    }
    let target = pick_target(facts.accounts, &me.key);
    let ready = facts.cycle == CycleView::Ready;
    if me.over() || ready {
        return at_point(facts, me, target, ready);
    }
    if target.is_none() {
        return Vec::new();
    }
    before_point(facts, me)
}

fn at_point(facts: &Facts<'_>, me: &View, target: Option<&View>, ready: bool) -> Vec<Decision> {
    let Some(target) = target else {
        return no_target(facts, me);
    };
    let mut out = Vec::new();
    if matches!(facts.cycle, CycleView::Estimating | CycleView::Asked) {
        out.push(Decision::CutOffRest);
    }
    let reason = if ready {
        CountdownReason::HandoversReady
    } else {
        CountdownReason::SwitchPoint
    };
    out.push(Decision::StartCountdown {
        to: target.key.clone(),
        reason,
    });
    out
}

fn no_target(facts: &Facts<'_>, me: &View) -> Vec<Decision> {
    let window = Window {
        account: me.key.clone(),
        resets_at_ms: me.resets_at_ms,
    };
    if facts.told == Some(&window) {
        Vec::new()
    } else {
        vec![Decision::NotifyNoTarget { window }]
    }
}

fn before_point(facts: &Facts<'_>, me: &View) -> Vec<Decision> {
    let Some(percent) = me.percent else {
        return Vec::new();
    };
    match me.rules.handover {
        HandoverMode::Off => Vec::new(),
        HandoverMode::At(at) => fixed(facts, percent, at),
        HandoverMode::Auto => auto(facts, percent),
    }
}

fn new_leads(facts: &Facts<'_>) -> Vec<Decision> {
    facts
        .estimates
        .filter(|e| !e.new_leads.is_empty())
        .map(|e| Decision::AddLeads(e.new_leads.clone()))
        .into_iter()
        .collect()
}

fn fixed(facts: &Facts<'_>, percent: f64, at: f64) -> Vec<Decision> {
    if percent < at {
        return Vec::new();
    }
    match facts.cycle {
        CycleView::None => vec![
            Decision::StartCycle {
                mode: CycleMode::Fixed,
                ask_at: Some(at),
            },
            Decision::AskAll,
        ],
        CycleView::Estimating => vec![Decision::AskAll],
        CycleView::Asked => new_leads(facts),
        CycleView::Foreign | CycleView::Ready => Vec::new(),
    }
}

fn auto(facts: &Facts<'_>, percent: f64) -> Vec<Decision> {
    let Some(estimates) = facts.estimates else {
        return Vec::new();
    };
    let at_ask = percent >= estimates.ask_at;
    match facts.cycle {
        CycleView::None if at_ask && estimates.has_leads => vec![
            Decision::StartCycle {
                mode: CycleMode::Auto,
                ask_at: None,
            },
            Decision::AskAll,
        ],
        CycleView::None if !estimates.ask_estimates.is_empty() => vec![Decision::StartCycle {
            mode: CycleMode::Auto,
            ask_at: None,
        }],
        CycleView::Estimating if at_ask => vec![Decision::AskAll],
        CycleView::Estimating => {
            let mut out = Vec::new();
            if !estimates.ask_estimates.is_empty() {
                out.push(Decision::StartEstimates(estimates.ask_estimates.clone()));
            }
            out.extend(new_leads(facts));
            out
        }
        CycleView::Asked => new_leads(facts),
        CycleView::None | CycleView::Foreign | CycleView::Ready => Vec::new(),
    }
}

/// The grace ended: switch when the checks hold, else stand down.
fn on_countdown(facts: &Facts<'_>, countdown: &Countdown) -> Vec<Decision> {
    if facts.now_ms < countdown.ends_at_ms {
        return Vec::new();
    }
    let stand_down = |reason| vec![Decision::StandDown { reason }];
    if facts.active != Some(countdown.from.as_str()) {
        return stand_down(StandDownReason::AccountChanged);
    }
    let Some(from) = view_of(facts, &countdown.from) else {
        return stand_down(StandDownReason::AccountChanged);
    };
    if !from.rules.auto_switch {
        return stand_down(StandDownReason::RuleOff);
    }
    if !(from.over() || facts.cycle == CycleView::Ready) {
        return stand_down(StandDownReason::BelowPoint);
    }
    if !view_of(facts, &countdown.to).is_some_and(View::has_room) {
        return stand_down(StandDownReason::TargetFull);
    }
    vec![Decision::Switch {
        to: countdown.to.clone(),
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(key: &str, percent: f64, resets: i64) -> View {
        View {
            key: key.to_owned(),
            label: key.to_uppercase(),
            can_receive: true,
            rules: Rules {
                auto_switch: true,
                switch_at: 98.0,
                handover: HandoverMode::Off,
                can_receive: true,
            },
            percent: Some(percent),
            critical: false,
            resets_at_ms: Some(resets),
        }
    }

    fn lead(thread: &str) -> LeadKey {
        LeadKey {
            board: "rotp".to_owned(),
            thread: thread.to_owned(),
        }
    }

    fn estimates(ask_at: f64) -> Estimates {
        Estimates {
            ask_at,
            has_leads: true,
            ask_estimates: Vec::new(),
            new_leads: Vec::new(),
        }
    }

    struct Setup {
        accounts: Vec<View>,
        active: Option<String>,
        estimates: Option<Estimates>,
        cycle: CycleView,
        hold_offs: Vec<HoldOff>,
        countdown: Option<Countdown>,
        told: Option<Window>,
        now_ms: i64,
    }

    fn setup(me: View, other: View) -> Setup {
        Setup {
            active: Some(me.key.clone()),
            accounts: vec![me, other],
            estimates: None,
            cycle: CycleView::None,
            hold_offs: Vec::new(),
            countdown: None,
            told: None,
            now_ms: 1_000,
        }
    }

    fn run(s: &Setup) -> Vec<Decision> {
        decide(&Facts {
            accounts: &s.accounts,
            active: s.active.as_deref(),
            estimates: s.estimates.as_ref(),
            cycle: s.cycle,
            hold_offs: &s.hold_offs,
            countdown: s.countdown.as_ref(),
            told: s.told.as_ref(),
            now_ms: s.now_ms,
        })
    }

    fn countdown_to(to: &str) -> Decision {
        Decision::StartCountdown {
            to: to.to_owned(),
            reason: CountdownReason::SwitchPoint,
        }
    }

    #[test]
    fn nothing_happens_below_the_switch_point() {
        let s = setup(view("a", 60.0, 10), view("b", 10.0, 20));
        assert!(run(&s).is_empty());
    }

    #[test]
    fn the_switch_point_starts_a_countdown_to_the_other_account() {
        let s = setup(view("a", 98.0, 10), view("b", 10.0, 20));
        assert_eq!(run(&s), vec![countdown_to("b")]);
    }

    #[test]
    fn a_critical_severity_or_a_full_limit_counts_as_the_point() {
        let mut critical = view("a", 50.0, 10);
        critical.critical = true;
        assert_eq!(
            run(&setup(critical, view("b", 10.0, 20))),
            vec![countdown_to("b")]
        );
        let mut full = view("a", 100.0, 10);
        full.rules.switch_at = 100.0;
        assert_eq!(
            run(&setup(full, view("b", 10.0, 20))),
            vec![countdown_to("b")]
        );
    }

    #[test]
    fn an_account_with_the_rule_off_never_switches() {
        let mut me = view("a", 99.0, 10);
        me.rules.auto_switch = false;
        assert!(run(&setup(me, view("b", 10.0, 20))).is_empty());
    }

    #[test]
    fn the_target_has_the_lowest_percent_then_the_soonest_reset() {
        let accounts = [
            view("a", 99.0, 10),
            view("b", 30.0, 500),
            view("c", 30.0, 200),
            view("d", 5.0, 900),
        ];
        assert_eq!(pick_target(&accounts, "a").unwrap().key, "d");
        assert_eq!(pick_target(&accounts[..3], "a").unwrap().key, "c");
    }

    #[test]
    fn a_target_must_receive_and_be_under_its_own_switch_point() {
        let mut full = view("b", 98.0, 20);
        full.rules.switch_at = 98.0;
        let mut shut = view("c", 1.0, 20);
        shut.can_receive = false;
        let mut unread = view("d", 1.0, 20);
        unread.percent = None;
        let accounts = [view("a", 99.0, 10), full, shut, unread];
        assert!(pick_target(&accounts, "a").is_none());
    }

    #[test]
    fn no_target_is_told_once_per_window() {
        let mut s = setup(view("a", 99.0, 10), view("b", 99.0, 20));
        let window = Window {
            account: "a".to_owned(),
            resets_at_ms: Some(10),
        };
        assert_eq!(
            run(&s),
            vec![Decision::NotifyNoTarget {
                window: window.clone()
            }]
        );
        s.told = Some(window);
        assert!(run(&s).is_empty());
        s.accounts[0].resets_at_ms = Some(99);
        assert_eq!(run(&s).len(), 1);
    }

    #[test]
    fn handover_off_goes_straight_to_the_countdown_even_with_leads() {
        let mut s = setup(view("a", 98.5, 10), view("b", 10.0, 20));
        s.estimates = Some(estimates(95.0));
        assert_eq!(run(&s), vec![countdown_to("b")]);
    }

    #[test]
    fn fixed_handover_starts_a_cycle_at_its_percent_and_asks_everyone() {
        let mut me = view("a", 90.0, 10);
        me.rules.handover = HandoverMode::At(90.0);
        let mut s = setup(me, view("b", 10.0, 20));
        assert_eq!(
            run(&s),
            vec![
                Decision::StartCycle {
                    mode: CycleMode::Fixed,
                    ask_at: Some(90.0)
                },
                Decision::AskAll
            ]
        );
        s.accounts[0].percent = Some(89.9);
        assert!(run(&s).is_empty());
    }

    #[test]
    fn fixed_handover_with_an_asked_cycle_only_adds_new_leads() {
        let mut me = view("a", 91.0, 10);
        me.rules.handover = HandoverMode::At(90.0);
        let mut s = setup(me, view("b", 10.0, 20));
        s.cycle = CycleView::Asked;
        assert!(run(&s).is_empty());
        let mut e = estimates(90.0);
        e.new_leads = vec![lead("coordinator")];
        s.estimates = Some(e);
        assert_eq!(run(&s), vec![Decision::AddLeads(vec![lead("coordinator")])]);
    }

    #[test]
    fn handover_without_a_target_starts_nothing() {
        let mut me = view("a", 91.0, 10);
        me.rules.handover = HandoverMode::At(90.0);
        let s = setup(me, view("b", 99.0, 20));
        assert!(run(&s).is_empty());
    }

    fn auto_setup(percent: f64) -> Setup {
        let mut me = view("a", percent, 10);
        me.rules.handover = HandoverMode::Auto;
        setup(me, view("b", 10.0, 20))
    }

    #[test]
    fn auto_handover_asks_for_estimates_when_the_estimator_says_near() {
        let mut s = auto_setup(90.0);
        let mut e = estimates(95.7);
        e.ask_estimates = vec![lead("coordinator")];
        s.estimates = Some(e);
        assert_eq!(
            run(&s),
            vec![Decision::StartCycle {
                mode: CycleMode::Auto,
                ask_at: None
            }]
        );
        s.estimates = Some(estimates(95.7));
        assert!(run(&s).is_empty());
    }

    #[test]
    fn auto_handover_re_asks_stale_leads_while_estimating() {
        let mut s = auto_setup(92.0);
        s.cycle = CycleView::Estimating;
        let mut e = estimates(95.7);
        e.ask_estimates = vec![lead("coordinator")];
        e.new_leads = vec![lead("late")];
        s.estimates = Some(e);
        assert_eq!(
            run(&s),
            vec![
                Decision::StartEstimates(vec![lead("coordinator")]),
                Decision::AddLeads(vec![lead("late")])
            ]
        );
    }

    #[test]
    fn auto_handover_asks_everyone_at_the_ask_point() {
        let mut s = auto_setup(95.7);
        s.cycle = CycleView::Estimating;
        s.estimates = Some(estimates(95.7));
        assert_eq!(run(&s), vec![Decision::AskAll]);
        s.cycle = CycleView::None;
        assert_eq!(
            run(&s),
            vec![
                Decision::StartCycle {
                    mode: CycleMode::Auto,
                    ask_at: None
                },
                Decision::AskAll
            ]
        );
    }

    #[test]
    fn auto_handover_with_no_working_lead_starts_no_cycle() {
        let mut s = auto_setup(96.0);
        let mut e = estimates(95.7);
        e.has_leads = false;
        s.estimates = Some(e);
        assert!(run(&s).is_empty());
    }

    #[test]
    fn a_ready_cycle_starts_the_countdown_below_the_switch_point() {
        let mut s = auto_setup(96.0);
        s.cycle = CycleView::Ready;
        assert_eq!(
            run(&s),
            vec![Decision::StartCountdown {
                to: "b".to_owned(),
                reason: CountdownReason::HandoversReady
            }]
        );
    }

    #[test]
    fn the_switch_point_with_an_open_cycle_cuts_the_rest_off_first() {
        for open in [CycleView::Estimating, CycleView::Asked] {
            let mut s = auto_setup(98.0);
            s.cycle = open;
            assert_eq!(run(&s), vec![Decision::CutOffRest, countdown_to("b")]);
        }
    }

    #[test]
    fn a_foreign_open_cycle_does_not_stop_the_countdown() {
        let mut s = auto_setup(98.0);
        s.cycle = CycleView::Foreign;
        assert_eq!(run(&s), vec![countdown_to("b")]);
    }

    fn running(s: &mut Setup, ends: i64) {
        s.countdown = Some(Countdown {
            from: "a".to_owned(),
            to: "b".to_owned(),
            ends_at_ms: ends,
        });
    }

    #[test]
    fn a_running_countdown_waits_for_its_end() {
        let mut s = setup(view("a", 99.0, 10), view("b", 10.0, 20));
        running(&mut s, 5_000);
        assert!(run(&s).is_empty());
    }

    #[test]
    fn the_countdown_end_switches_when_every_check_holds() {
        let mut s = setup(view("a", 99.0, 10), view("b", 10.0, 20));
        running(&mut s, 1_000);
        assert_eq!(run(&s), vec![Decision::Switch { to: "b".to_owned() }]);
    }

    #[test]
    fn the_countdown_end_switches_on_a_ready_cycle_below_the_point() {
        let mut s = setup(view("a", 90.0, 10), view("b", 10.0, 20));
        s.cycle = CycleView::Ready;
        running(&mut s, 1_000);
        assert_eq!(run(&s), vec![Decision::Switch { to: "b".to_owned() }]);
    }

    fn stood_down(reason: StandDownReason) -> Vec<Decision> {
        vec![Decision::StandDown { reason }]
    }

    #[test]
    fn the_countdown_end_stands_down_when_the_use_dropped() {
        let mut s = setup(view("a", 80.0, 10), view("b", 10.0, 20));
        running(&mut s, 1_000);
        assert_eq!(run(&s), stood_down(StandDownReason::BelowPoint));
    }

    #[test]
    fn the_countdown_end_stands_down_when_the_target_is_full() {
        let mut s = setup(view("a", 99.0, 10), view("b", 98.0, 20));
        running(&mut s, 1_000);
        assert_eq!(run(&s), stood_down(StandDownReason::TargetFull));
        s.accounts[1].percent = Some(5.0);
        s.accounts[1].can_receive = false;
        assert_eq!(run(&s), stood_down(StandDownReason::TargetFull));
    }

    #[test]
    fn the_countdown_end_stands_down_when_claude_changed_account() {
        let mut s = setup(view("a", 99.0, 10), view("b", 10.0, 20));
        running(&mut s, 1_000);
        s.active = Some("b".to_owned());
        assert_eq!(run(&s), stood_down(StandDownReason::AccountChanged));
        s.active = None;
        assert_eq!(run(&s), stood_down(StandDownReason::AccountChanged));
    }

    #[test]
    fn the_countdown_end_stands_down_when_the_rule_was_turned_off() {
        let mut s = setup(view("a", 99.0, 10), view("b", 10.0, 20));
        running(&mut s, 1_000);
        s.accounts[0].rules.auto_switch = false;
        assert_eq!(run(&s), stood_down(StandDownReason::RuleOff));
    }

    #[test]
    fn a_hold_off_stops_the_rule_until_the_use_falls_five_points_under() {
        let mut s = setup(view("a", 99.0, 10), view("b", 10.0, 20));
        s.hold_offs = vec![HoldOff::after_cancel(&s.accounts[0])];
        assert!(run(&s).is_empty());
        s.accounts[0].percent = Some(94.0);
        assert!(run(&s).is_empty());
        s.accounts[0].percent = Some(93.0);
        assert_eq!(
            run(&s),
            vec![Decision::ClearHoldOff {
                account: "a".to_owned()
            }]
        );
    }

    #[test]
    fn a_hold_off_ends_when_the_limit_resets() {
        let mut s = setup(view("a", 99.0, 10), view("b", 10.0, 20));
        s.hold_offs = vec![HoldOff::after_cancel(&s.accounts[0])];
        s.accounts[0].resets_at_ms = Some(7_000_000);
        let out = run(&s);
        assert_eq!(
            out[0],
            Decision::ClearHoldOff {
                account: "a".to_owned()
            }
        );
        s.now_ms = 11;
        s.accounts[0].resets_at_ms = Some(10);
        assert_eq!(
            run(&s).len(),
            2,
            "the reset time passed: cleared, then the rule runs"
        );
    }

    #[test]
    fn a_hold_off_ends_when_the_owner_turns_the_rule_off_and_on() {
        let mut s = setup(view("a", 99.0, 10), view("b", 10.0, 20));
        s.hold_offs = vec![HoldOff::after_cancel(&s.accounts[0])];
        s.accounts[0].rules.auto_switch = false;
        assert_eq!(
            run(&s),
            vec![Decision::ClearHoldOff {
                account: "a".to_owned()
            }]
        );
    }

    #[test]
    fn a_hold_off_of_another_account_does_not_hold_this_one() {
        let mut s = setup(view("a", 99.0, 10), view("b", 10.0, 20));
        s.hold_offs = vec![HoldOff::after_cancel(&s.accounts[1])];
        assert_eq!(run(&s), vec![countdown_to("b")]);
    }

    #[test]
    fn an_unknown_active_account_does_nothing() {
        let mut s = setup(view("a", 99.0, 10), view("b", 10.0, 20));
        s.active = None;
        assert!(run(&s).is_empty());
    }
}
