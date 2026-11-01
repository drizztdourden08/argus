//! The toast fallback: the tray balloon takes over when the platform refuses a toast.
//! Keep rule: S2.
// test-tag: standard

use std::sync::Arc;

use argus_native::{NotifyError, RecordingNotifier};
use argus_notify::{Delivery, Level, ToastRequest};

use crate::harness::ScriptedTray;

fn request() -> ToastRequest {
    ToastRequest {
        title: "T".into(),
        text: "x".into(),
        level: Level::Ping,
        url: Some("http://argus.localhost:47615/#notifications".into()),
        tag: "abc".into(),
        sound: true,
    }
}

fn refused(reason: &str, tray: &Arc<ScriptedTray>) -> Delivery {
    let notifier = RecordingNotifier::failing(NotifyError::Platform(reason.into()));
    Delivery::new(Arc::new(notifier), Some(tray.clone()))
}

#[test]
fn a_toast_the_platform_accepts_is_shown_and_the_tray_is_not_asked() {
    let tray = Arc::new(ScriptedTray::default());
    let notifier = Arc::new(RecordingNotifier::new());
    let delivery = Delivery::new(notifier.clone(), Some(tray.clone()));
    assert_eq!(
        delivery.deliver(&request()),
        argus_notify::Delivered::Shown(None)
    );
    assert_eq!(notifier.shown().len(), 1);
    assert_eq!(tray.asked().len(), 0);
}

#[test]
fn a_refused_toast_falls_back_to_the_tray_balloon_when_there_is_one() {
    let tray = Arc::new(ScriptedTray::default());
    tray.answer(vec![true]);
    let delivered = refused("PowerShell exit 2", &tray).deliver(&request());
    assert_eq!(
        delivered,
        argus_notify::Delivered::Shown(Some("shown as a tray balloon (PowerShell exit 2)".into()))
    );
    assert_eq!(tray.asked(), [("T".to_owned(), "x".to_owned())]);
}

#[test]
fn a_balloon_that_fails_or_a_missing_tray_leaves_the_reason() {
    let tray = Arc::new(ScriptedTray::default());
    tray.answer(vec![false]);
    let delivered = refused("PowerShell exit 2", &tray).deliver(&request());
    assert_eq!(
        delivered,
        argus_notify::Delivered::Failed("PowerShell exit 2".into())
    );
    let notifier = RecordingNotifier::failing(NotifyError::Platform("no WinRT".into()));
    let bare = Delivery::new(Arc::new(notifier), None);
    assert_eq!(
        bare.deliver(&request()),
        argus_notify::Delivered::Failed("no WinRT".into())
    );
}

#[test]
fn an_invalid_toast_is_a_reason_not_a_panic() {
    let tray = Arc::new(ScriptedTray::default());
    let notifier = Arc::new(RecordingNotifier::new());
    let delivery = Delivery::new(notifier, Some(tray));
    let empty = ToastRequest {
        title: String::new(),
        text: String::new(),
        ..request()
    };
    assert_eq!(
        delivery.deliver(&empty),
        argus_notify::Delivered::Failed("a toast needs a title or a text".into())
    );
}
