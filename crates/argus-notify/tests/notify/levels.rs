//! Levels, config switches, a failing toast, and the activity lines.

use argus_native::NotifyError;
use argus_notify::{ActivityLevel, By, Level, NotifyConfig, PingRequest, ToastState};

use crate::harness::{Harness, from_lead};

fn at(level: Level, text: &str) -> PingRequest {
    PingRequest {
        level,
        ..PingRequest::new(text)
    }
}

#[test]
fn info_never_asks_for_a_toast_and_writes_no_activity_line_ping_and_urgent_do() {
    let h = Harness::new();
    let info = h.notify.send(PingRequest {
        level: Level::Info,
        ..from_lead("fyi")
    });
    assert_eq!(h.shown().len(), 0);
    assert_eq!(info.toast, ToastState::Skipped);
    assert_eq!(info.why.as_deref(), Some("info is recorded only"));
    assert_eq!(info.notification.expect("recorded").url, None);
    assert_eq!(h.activity.lines().len(), 0);
    h.minute();
    h.notify.send(at(Level::Ping, "look"));
    h.minute();
    h.notify.send(PingRequest {
        level: Level::Urgent,
        ..from_lead("now")
    });
    let urgent: Vec<bool> = h.shown().iter().map(|toast| toast.urgent).collect();
    assert_eq!(urgent, [false, true]);
}

#[test]
fn os_off_records_without_a_toast_and_quiet_levels_skip_just_those_levels() {
    let off = Harness::with(NotifyConfig {
        os: false,
        ..NotifyConfig::default()
    });
    let a = off.notify.send(at(Level::Urgent, "x"));
    assert_eq!(a.toast, ToastState::Skipped);
    assert_eq!(
        a.why.as_deref(),
        Some("OS notifications are off (config notify.os)")
    );
    assert_eq!(off.shown().len(), 0);
    assert_eq!(off.store.rows().len(), 1);
    let quiet = Harness::with(NotifyConfig {
        quiet_levels: vec!["ping".into()],
        ..NotifyConfig::default()
    });
    let p = quiet.notify.send(at(Level::Ping, "x"));
    let u = quiet.notify.send(PingRequest {
        thread: Some("t2".into()),
        ..at(Level::Urgent, "y")
    });
    assert_eq!(p.toast, ToastState::Skipped);
    assert_eq!(
        p.why.as_deref(),
        Some("ping is a quiet level (config notify.quietLevels)")
    );
    assert_eq!(u.toast, ToastState::Shown);
    let texts: Vec<_> = quiet
        .shown()
        .into_iter()
        .map(|toast| toast.texts[1].clone())
        .collect();
    assert_eq!(texts, ["y"]);
}

#[test]
fn a_toast_that_fails_is_recorded_as_failed_with_its_reason_a_warning_and_never_thrown() {
    let h = Harness::new();
    h.notifier.script(vec![
        Err(NotifyError::Platform("PowerShell exit 2: no WinRT".into())),
        Err(NotifyError::Platform("sender blew up".into())),
        Err(NotifyError::Platform("x".into())),
    ]);
    h.tray.answer(vec![false, false, true]);
    let a = h.notify.send(from_lead("one"));
    let b = h.notify.send(from_lead("two"));
    let c = h.notify.send(from_lead("three"));
    assert!(a.is_recorded() && b.is_recorded());
    assert_eq!(
        (a.toast, a.why.as_deref()),
        (ToastState::Failed, Some("PowerShell exit 2: no WinRT"))
    );
    assert_eq!(
        (b.toast, b.why.as_deref()),
        (ToastState::Failed, Some("sender blew up"))
    );
    assert_eq!(
        (c.toast, c.why.as_deref()),
        (ToastState::Shown, Some("shown as a tray balloon (x)"))
    );
    let lines = h.activity.lines();
    let levels: Vec<_> = lines.iter().map(|line| line.level).collect();
    assert_eq!(
        levels,
        [
            ActivityLevel::Warn,
            ActivityLevel::Warn,
            ActivityLevel::Info
        ]
    );
    assert!(
        lines[0]
            .text
            .ends_with("no toast: failed, PowerShell exit 2: no WinRT)")
    );
    assert!(
        lines[2]
            .text
            .ends_with("(toast shown as a tray balloon (x))")
    );
    assert_eq!(
        h.tray.asked()[0],
        ("rotp: lead".to_owned(), "one".to_owned())
    );
}

#[test]
fn send_never_fails_even_with_the_database_closed() {
    let h = Harness::new();
    h.store.set_broken(true);
    let out = h.ping("x");
    assert!(!out.is_recorded());
    assert_eq!(out.toast, ToastState::Failed);
    assert_eq!(
        out.why.as_deref(),
        Some("the notification could not be stored")
    );
    let lines = h.activity.lines();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].text, "notification not recorded");
    assert_eq!(lines[0].level, ActivityLevel::Warn);
    assert_eq!(h.events.pushed().len(), 0);
}

#[test]
fn every_ping_and_urgent_is_one_activity_line_that_says_who_and_what_became_of_the_toast() {
    let h = Harness::new();
    h.notify.send(PingRequest {
        title: Some("Rotp: lead".into()),
        ..from_lead("B1 ready")
    });
    h.minute();
    h.notify.send(PingRequest {
        title: Some("Argus: test".into()),
        by: By::Page,
        ..PingRequest::new("hi")
    });
    h.minute();
    h.notify.send(PingRequest {
        title: Some("Rotp asks".into()),
        by: By::Auto,
        level: Level::Urgent,
        ..from_lead("q?")
    });
    h.minute();
    h.notify.send(PingRequest {
        title: Some("Rotp".into()),
        level: Level::Urgent,
        ..from_lead("now")
    });
    let lines = h.activity.lines();
    let texts: Vec<_> = lines.iter().map(|line| line.text.as_str()).collect();
    assert_eq!(
        texts,
        [
            "ping from rotp/lead: Rotp: lead: B1 ready (toast shown)",
            "ping from the page: Argus: test: hi (toast shown)",
            "automatic ping: Rotp asks: q? (toast shown)",
            "urgent ping from rotp/lead: Rotp: now (toast shown)",
        ]
    );
    let who: Vec<_> = lines
        .iter()
        .map(|line| (line.project.as_deref(), line.thread.as_deref()))
        .collect();
    assert_eq!(who[0], (Some("rotp"), Some("lead")));
    assert_eq!(who[1], (None, None));
}
