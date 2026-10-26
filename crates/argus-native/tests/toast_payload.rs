//! The toast payload through the public API, with the recording notifier as the platform.

use argus_native::{Level, Notifier, NotifyError, Payload, RecordingNotifier, Toast};

const OPEN_PAGE: &str = "http://argus.localhost:47615/#notifications";

fn xml_of(toast: &Toast) -> String {
    Payload::build(toast).expect("a valid toast").xml
}

fn tag_names(xml: &str) -> Vec<String> {
    xml.split('<')
        .skip(1)
        .map(|part| {
            part.chars()
                .take_while(|c| c.is_ascii_alphabetic() || *c == '/')
                .collect()
        })
        .collect()
}

#[test]
fn a_ping_is_a_toast_generic_with_two_lines_and_a_protocol_launch() {
    let toast = Toast::new("Rotp: lead", "B1 is ready to test").url("http://rotp.localhost:47615/");
    assert_eq!(
        xml_of(&toast),
        concat!(
            r#"<toast launch="http://rotp.localhost:47615/" activationType="protocol">"#,
            r#"<visual><binding template="ToastGeneric">"#,
            "<text>Rotp: lead</text><text>B1 is ready to test</text>",
            "</binding></visual></toast>"
        )
    );
}

#[test]
fn an_urgent_toast_is_a_reminder_with_open_and_dismiss_buttons() {
    let xml = xml_of(&Toast::new("T", "x").level(Level::Urgent).url(OPEN_PAGE));
    assert!(xml.starts_with(&format!(
        r#"<toast launch="{OPEN_PAGE}" activationType="protocol" scenario="reminder">"#
    )));
    assert!(xml.contains(&format!(
        r#"<actions><action content="Open" arguments="{OPEN_PAGE}" activationType="protocol"/><action content="Dismiss" arguments="dismiss" activationType="system"/></actions>"#
    )));
    assert!(!xml.contains("<audio"));
}

#[test]
fn an_urgent_toast_without_an_address_has_only_the_dismiss_button() {
    let xml = xml_of(&Toast::new("T", "x").level(Level::Urgent));
    assert!(xml.contains(r#"<actions><action content="Dismiss""#));
    assert!(!xml.contains("Open"));
    assert!(!xml.contains("launch="));
}

#[test]
fn ping_and_info_have_no_scenario_and_no_actions() {
    for level in [Level::Info, Level::Ping] {
        let xml = xml_of(&Toast::new("T", "x").level(level));
        assert!(!xml.contains("scenario") && !xml.contains("<actions"));
    }
}

#[test]
fn sound_false_is_a_silent_audio_element_for_every_level() {
    for level in [Level::Info, Level::Ping, Level::Urgent] {
        let silent = xml_of(&Toast::new("T", "x").level(level).sound(false));
        assert!(silent.ends_with(r#"<audio silent="true"/></toast>"#));
        assert!(!xml_of(&Toast::new("T", "x").level(level).sound(true)).contains("<audio"));
    }
}

#[test]
fn special_characters_are_escaped_and_an_injection_attempt_stays_text() {
    let evil = "a & b < c > d \"q\" 's' </text></binding></visual><actions><action content=\"Run\" arguments=\"calc.exe\" activationType=\"protocol\"/></actions><toast launch=\"file:///c:/x\">\u{7}\u{0} end";
    let xml = xml_of(&Toast::new("<b>\"T\"</b>", evil).url("http://x.localhost:1/?a=b&c=d"));
    assert!(!xml.contains('\u{7}') && !xml.contains('\u{0}'));
    assert_eq!(
        tag_names(&xml),
        [
            "toast", "visual", "binding", "text", "/text", "text", "/text", "/binding", "/visual",
            "/toast"
        ],
        "no element was smuggled in"
    );
    assert!(xml.contains("&lt;b&gt;&quot;T&quot;&lt;/b&gt;"));
    assert!(xml.contains(
        "a &amp; b &lt; c &gt; d &quot;q&quot; &apos;s&apos; &lt;/text&gt;&lt;/binding&gt;"
    ));
    assert!(xml.contains(r#"launch="http://x.localhost:1/?a=b&amp;c=d""#));
}

#[test]
fn title_and_text_are_cleaned_and_cut() {
    let xml = xml_of(&Toast::new("a\r\nb\tc", "x".repeat(300)));
    assert!(xml.contains("<text>a b c</text>"));
    let long = format!("<text>{}\u{2026}</text>", "x".repeat(239));
    assert!(xml.contains(&long));
}

#[test]
fn an_empty_title_leaves_a_single_line_and_nothing_at_all_is_refused() {
    let xml = xml_of(&Toast::new("  ", "only text"));
    assert_eq!(xml.matches("<text>").count(), 1);
    assert_eq!(
        Payload::build(&Toast::new("\u{200b}", " ")),
        Err(NotifyError::Empty)
    );
}

#[test]
fn only_plain_http_and_https_addresses_open_from_a_click() {
    for bad in [
        "file:///c:/windows/system32/calc.exe",
        "calc.exe",
        "javascript:alert(1)",
        "http://",
        "http://x/a b",
        "http://x/\"onclick",
        "http://x/<y>",
    ] {
        let result = Payload::build(&Toast::new("T", "x").url(bad));
        assert!(matches!(result, Err(NotifyError::InvalidUrl(_))), "{bad}");
    }
    let too_long = format!("http://x/{}", "a".repeat(300));
    assert!(Payload::build(&Toast::new("T", "x").url(too_long)).is_err());
    assert!(Payload::build(&Toast::new("T", "x").url("HTTPS://Argus.localhost/")).is_ok());
}

#[test]
fn the_tag_travels_with_the_payload_and_is_checked() {
    let payload = Payload::build(&Toast::new("T", "x").tag("0123456789ab")).expect("valid");
    assert_eq!(payload.tag.as_deref(), Some("0123456789ab"));
    assert_eq!(
        Payload::build(&Toast::new("T", "x")).expect("valid").tag,
        None
    );
    for bad in [String::new(), "t".repeat(65)] {
        let result = Payload::build(&Toast::new("T", "x").tag(bad));
        assert!(matches!(result, Err(NotifyError::InvalidTag(_))));
    }
}

#[test]
fn the_recording_notifier_keeps_what_was_shown_in_order() {
    let notifier = RecordingNotifier::new();
    notifier.show(&Toast::new("One", "a")).expect("shown");
    notifier
        .show(&Toast::new("Two", "b").url(OPEN_PAGE))
        .expect("shown");
    let shown = notifier.shown();
    assert_eq!(shown.len(), 2);
    assert!(shown[0].xml.contains("<text>One</text>"));
    assert!(shown[1].xml.contains(OPEN_PAGE));
}

#[test]
fn an_invalid_toast_is_refused_by_the_fake_like_by_the_real_notifier() {
    let notifier = RecordingNotifier::new();
    let result = notifier.show(&Toast::new("T", "x").url("ftp://x/"));
    assert!(matches!(result, Err(NotifyError::InvalidUrl(_))));
    assert_eq!(notifier.shown().len(), 0);
}

#[test]
fn a_failing_platform_is_an_error_not_a_panic() {
    let notifier = RecordingNotifier::failing(NotifyError::Platform("no shell".into()));
    let result = notifier.show(&Toast::new("T", "x"));
    assert_eq!(result, Err(NotifyError::Platform("no shell".into())));
    assert_eq!(notifier.shown().len(), 0);
    assert_eq!(
        result.expect_err("failed").to_string(),
        "the toast could not be shown: no shell"
    );
}

#[test]
fn the_notifier_is_usable_as_a_shared_trait_object() {
    let notifier: std::sync::Arc<dyn Notifier> = std::sync::Arc::new(RecordingNotifier::new());
    assert!(notifier.show(&Toast::new("T", "x")).is_ok());
}
