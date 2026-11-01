//! Sound and toast per level, `toastInfo`, and changing the settings from the page.
//! Keep rule: S2.
// test-tag: standard

use argus_notify::{
    Level, NotifyConfig, PerLevel, PingRequest, Settings, SettingsPatch, ToastState, settings_of,
};

use crate::harness::{Harness, ROTP};

fn at(level: Level, text: &str, thread: &str) -> PingRequest {
    PingRequest {
        level,
        thread: Some(thread.into()),
        ..PingRequest::new(text)
    }
}

fn sounds(ping: Option<bool>, urgent: Option<bool>) -> PerLevel<Option<bool>> {
    PerLevel {
        info: None,
        ping,
        urgent,
    }
}

#[test]
fn defaults_info_has_no_toast_ping_a_silent_toast_urgent_a_toast_with_the_system_sound() {
    let defaults = Settings {
        os: true,
        toast: PerLevel {
            info: false,
            ping: true,
            urgent: true,
        },
        sound: PerLevel {
            info: false,
            ping: false,
            urgent: true,
        },
    };
    assert_eq!(settings_of(&NotifyConfig::default()), defaults);
    let h = Harness::new();
    h.notify.send(at(Level::Ping, "a", "t"));
    h.minute();
    h.notify.send(at(Level::Urgent, "b", "t"));
    let silent: Vec<bool> = h.shown().iter().map(|toast| toast.silent).collect();
    assert_eq!(silent, [true, false]);
    assert_eq!(h.notify.settings(), defaults);
}

#[test]
fn sound_per_level_true_plays_the_system_sound_false_is_silent_and_a_summary_follows_ping() {
    let h = Harness::with(NotifyConfig {
        sound: sounds(Some(true), Some(false)),
        ..NotifyConfig::default()
    });
    h.notify.send(at(Level::Ping, "a", "t"));
    h.notify.send(at(Level::Urgent, "b", "u"));
    for _ in 0..3 {
        h.notify.send(at(Level::Ping, "c", "t"));
    }
    let shown = h.shown();
    assert_eq!((shown[0].silent, shown[1].silent), (false, true));
    let summaries: Vec<bool> = shown
        .iter()
        .filter(|toast| toast.is_summary())
        .map(|toast| toast.silent)
        .collect();
    assert_eq!(
        summaries,
        [false],
        "the summary is a ping: it follows the ping sound"
    );
    let partial = Harness::with(NotifyConfig {
        sound: sounds(None, Some(false)),
        ..NotifyConfig::default()
    });
    assert_eq!(
        partial.notify.settings().sound,
        PerLevel {
            info: false,
            ping: false,
            urgent: false
        }
    );
}

#[test]
fn toast_info_gives_info_a_silent_toast_with_no_activity_line_and_quiet_levels_and_os_win() {
    let h = Harness::named(NotifyConfig {
        toast_info: true,
        ..NotifyConfig::default()
    });
    let out = h.notify.send(PingRequest {
        level: Level::Info,
        project: Some("rotp".into()),
        thread: Some("lead".into()),
        ..PingRequest::new("fyi")
    });
    assert_eq!(out.toast, ToastState::Shown);
    let shown = h.shown();
    assert_eq!(
        (shown.len(), shown[0].silent, shown[0].url.as_deref()),
        (1, true, Some(ROTP))
    );
    assert_eq!(
        out.notification.expect("recorded").url.as_deref(),
        Some(ROTP)
    );
    assert_eq!(h.activity.lines().len(), 0);
    let loud = Harness::with(NotifyConfig {
        toast_info: true,
        sound: PerLevel {
            info: Some(true),
            ping: None,
            urgent: None,
        },
        ..NotifyConfig::default()
    });
    loud.notify.send(at(Level::Info, "fyi", "t"));
    assert!(!loud.shown()[0].silent);
    let quiet = Harness::with(NotifyConfig {
        toast_info: true,
        quiet_levels: vec!["info".into()],
        ..NotifyConfig::default()
    });
    assert_eq!(
        quiet.notify.send(at(Level::Info, "x", "t")).toast,
        ToastState::Skipped
    );
    assert_eq!(quiet.shown().len(), 0);
    let off = Harness::with(NotifyConfig {
        toast_info: true,
        os: false,
        ..NotifyConfig::default()
    });
    assert_eq!(
        off.notify.send(at(Level::Info, "x", "t")).toast,
        ToastState::Skipped
    );
    assert_eq!(off.shown().len(), 0);
}

fn first_patch() -> SettingsPatch {
    SettingsPatch {
        toast: PerLevel {
            info: Some(true),
            ping: Some(false),
            urgent: None,
        },
        sound: PerLevel {
            info: Some(true),
            ping: Some(true),
            urgent: None,
        },
    }
}

fn second_patch() -> SettingsPatch {
    SettingsPatch {
        toast: PerLevel {
            info: Some(false),
            ping: Some(true),
            urgent: None,
        },
        sound: sounds(None, Some(false)),
    }
}

#[test]
fn configure_writes_toast_and_sound_per_level_to_the_config_and_saves_it_once() {
    let h = Harness::new();
    let out = h.notify.configure(first_patch()).expect("saved");
    let (toast, sound) = (out.toast, out.sound);
    assert_eq!((toast.info, toast.ping, toast.urgent), (true, false, true));
    assert_eq!((sound.info, sound.ping, sound.urgent), (true, true, true));
    let config = h.config.get();
    assert_eq!(config.quiet_levels, ["ping"]);
    assert!(config.toast_info);
    assert_eq!(config.sound, PerLevel::from_fn(|_| Some(true)));
    assert_eq!(h.config.saves(), 1, "config.json written once");
    assert_eq!(h.ping("x").toast, ToastState::Skipped);
    let info = h.notify.send(at(Level::Info, "y", "t"));
    assert_eq!(info.toast, ToastState::Shown);
}

#[test]
fn a_second_change_keeps_what_it_does_not_name_and_leaves_an_activity_line() {
    let h = Harness::new();
    h.notify.configure(first_patch()).expect("saved");
    h.notify.configure(second_patch()).expect("saved");
    let config = h.config.get();
    assert_eq!(config.quiet_levels.len(), 0);
    let toast = h.notify.settings().toast;
    assert_eq!((toast.info, toast.ping, toast.urgent), (false, true, true));
    assert_eq!(config.sound.urgent, Some(false));
    assert!(
        config.os && config.auto.handed && config.auto.ask,
        "the other keys stay"
    );
    let line = h.activity.lines().pop().expect("a line");
    assert_eq!(
        line.text,
        "notification settings changed from the page: info no toast, ping toast with sound, urgent toast silent"
    );
}

#[test]
fn a_config_that_cannot_be_written_changes_nothing_and_says_why() {
    let h = Harness::new();
    h.config.refuse_saves(true);
    let patch = SettingsPatch {
        toast: PerLevel {
            info: None,
            ping: Some(false),
            urgent: None,
        },
        ..SettingsPatch::default()
    };
    let error = h.notify.configure(patch).expect_err("refused");
    assert_eq!(error.to_string(), "disk full");
    assert_eq!(h.config.get().quiet_levels.len(), 0);
    assert!(h.notify.settings().toast.ping);
}
