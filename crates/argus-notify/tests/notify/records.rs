//! Records, docs, the click address, the cap of 50 and the list.

use argus_native::{TEXT_MAX, TITLE_MAX};
use argus_notify::{By, Filter, KEEP_ROWS, Level, PingRequest, ToastState};

use crate::harness::{Harness, ROTP, TAB, from_lead};

#[test]
fn a_ping_is_recorded_with_its_doc_the_project_link_and_the_toast_that_was_asked_for() {
    let h = Harness::new();
    let out = h.notify.send(PingRequest {
        title: Some("Rotp: lead".into()),
        level: Level::Ping,
        by: By::Thread,
        ..from_lead("B1 is ready to test")
    });
    assert!(out.is_recorded());
    assert_eq!(out.toast, ToastState::Shown);
    assert_eq!(out.id(), Some("n-1"));
    let note = out.notification.expect("recorded");
    let doc = serde_json::json!({
        "id": "n-1", "n": 1, "at": "2026-10-26T12:00:00.000Z", "level": "ping",
        "title": "Rotp: lead", "text": "B1 is ready to test", "project": "rotp",
        "thread": "lead", "by": "thread", "toast": "shown", "url": ROTP, "why": null,
    });
    assert_eq!(serde_json::to_value(&note).expect("serializes"), doc);
    assert_eq!(h.events.pushed(), [note]);
    let shown = h.shown();
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].texts, ["Rotp: lead", "B1 is ready to test"]);
    assert_eq!(shown[0].url.as_deref(), Some(ROTP));
    assert!(shown[0].silent && !shown[0].urgent);
    let tag = shown[0].tag.as_deref().expect("a tag");
    assert!(tag.len() == 12 && tag.chars().all(|c| c.is_ascii_hexdigit()));
}

/// The click target as Node's `toastXml` writes it: the page address in the `launch` attribute,
/// opened by protocol activation, on the project's alias host or else the Notifications tab.
#[test]
fn a_click_on_the_toast_opens_the_board_or_the_notifications_tab_by_protocol_activation() {
    let h = Harness::new();
    h.notify.send(from_lead("B1 is ready to test"));
    h.notify.send(PingRequest::new("no project here"));
    let xml: Vec<String> = h.notifier.shown().into_iter().map(|p| p.xml).collect();
    assert_eq!(xml.len(), 2);
    let board = format!("<toast launch=\"{ROTP}\" activationType=\"protocol\">");
    let tab = format!("<toast launch=\"{TAB}\" activationType=\"protocol\">");
    assert!(xml[0].starts_with(&board), "{}", xml[0]);
    assert!(xml[1].starts_with(&tab), "{}", xml[1]);
}

#[test]
fn defaults_and_cleaning_title_from_the_sender_plain_text_title_60_and_text_240() {
    let h = Harness::new();
    let first = h.notify.send(from_lead("hello"));
    let long = h.notify.send(PingRequest {
        title: Some("T".repeat(100)),
        level: Level::or_ping(Some("nonsense")),
        by: By::resolve(Some("nobody"), None),
        ..PingRequest::new(format!("line one\r\n\u{7}line two {}", "y".repeat(400)))
    });
    let (a, b) = (
        first.notification.expect("a"),
        long.notification.expect("b"),
    );
    assert_eq!((a.level, a.by), (Level::Ping, By::Thread));
    assert_eq!(a.title, "rotp: lead");
    assert_eq!(b.title.chars().count(), TITLE_MAX);
    assert!(b.title.ends_with('\u{2026}'));
    assert_eq!(b.text.chars().count(), TEXT_MAX);
    assert!(b.text.starts_with("line one line two y") && b.text.ends_with('\u{2026}'));
    assert_eq!(b.level, Level::Ping);
    assert_eq!(b.url.as_deref(), Some(TAB));
    let empty = h.ping("  \u{7} ");
    assert!(!empty.is_recorded());
    assert_eq!(empty.toast, ToastState::Skipped);
    assert_eq!(empty.why.as_deref(), Some("a notification needs text"));
    assert_eq!(h.store.rows().len(), 2, "an empty text is not recorded");
}

#[test]
fn the_click_address_is_only_ever_an_http_address_on_this_machine() {
    let h = Harness::new();
    let ask = |text: &str, project: Option<&str>, url: Option<&str>| {
        h.notify.send(PingRequest {
            project: project.map(str::to_owned),
            url: url.map(str::to_owned),
            ..PingRequest::new(text)
        })
    };
    ask("a", Some("portfolio"), None);
    ask("b", None, Some("http://rotp.localhost:47615/#x"));
    ask("c", None, Some("file:///c:/windows/system32/calc.exe"));
    ask("d", None, Some("http://evil.example.com/"));
    ask("e", Some("Not A Key"), None);
    let urls: Vec<_> = h
        .store
        .rows()
        .into_iter()
        .map(|note| note.url.clone())
        .collect();
    let tab = Some(TAB.to_owned());
    assert_eq!(
        urls,
        [
            tab.clone(),
            Some("http://rotp.localhost:47615/#x".into()),
            tab.clone(),
            tab.clone(),
            tab
        ]
    );
}

#[test]
fn the_store_keeps_the_last_50_and_start_writes_the_docs_again_on_a_reset() {
    let h = Harness::new();
    h.notify.start();
    assert_eq!(h.store.mirrors(), 1);
    for i in 1..=KEEP_ROWS + 7 {
        h.minute();
        h.notify.send(PingRequest {
            level: Level::Info,
            ..PingRequest::new(format!("n {i}"))
        });
    }
    let rows = h.store.rows();
    assert_eq!(rows.len(), KEEP_ROWS);
    assert_eq!((rows[0].n, rows[KEEP_ROWS - 1].n), (8, 57));
    assert!(rows.iter().all(|note| note.id == format!("n-{}", note.n)));
    h.notify.mirror();
    assert_eq!(h.store.mirrors(), 2);
    h.notify.start();
    assert_eq!(h.store.mirrors(), 2, "a second start does nothing");
    h.notify.stop();
    assert!(!h.notify.is_running());
}

#[test]
fn list_is_newest_first_filtered_by_level_and_project_and_at_most_50() {
    let h = Harness::new();
    let send = |text: &str, level: Level, project: &str| {
        h.notify.send(PingRequest {
            level,
            project: Some(project.into()),
            ..PingRequest::new(text)
        });
    };
    send("a", Level::Info, "rotp");
    h.minute();
    send("b", Level::Ping, "brock");
    send("c", Level::Urgent, "rotp");
    let texts = |filter: Filter| -> Vec<String> {
        h.notify
            .list(&filter)
            .expect("listed")
            .into_iter()
            .map(|note| note.text.clone())
            .collect()
    };
    assert_eq!(texts(Filter::default()), ["c", "b", "a"]);
    assert_eq!(
        texts(Filter {
            level: Some(Level::Urgent),
            ..Filter::default()
        }),
        ["c"]
    );
    let rotp = Filter {
        project: Some("rotp".into()),
        ..Filter::default()
    };
    assert_eq!(texts(rotp), ["c", "a"]);
    assert_eq!(
        texts(Filter {
            limit: 1,
            ..Filter::default()
        }),
        ["c"]
    );
    assert_eq!(
        texts(Filter {
            limit: 0,
            ..Filter::default()
        }),
        ["c"],
        "at least one"
    );
    assert_eq!(
        texts(Filter {
            limit: 500,
            ..Filter::default()
        })
        .len(),
        3
    );
}
