//! The toast XML of a ping and of an urgent toast loads into the real `WinRT` document, and the
//! notification carries its tag and group; nothing is shown. Ported from the last case of the
//! `toastScript` group of `notify.test.mjs` (the PowerShell script and its parse check have no
//! counterpart: the notifier is native code now).
//! Keep rule: E2.
// test-tag: e2e
#![cfg(windows)]

use argus_native::{GROUP, Level, Payload, Toast};
use windows::Data::Xml::Dom::XmlDocument;
use windows::UI::Notifications::ToastNotification;
use windows::core::HSTRING;

const TAG: &str = "abc123abc123";

/// What `WinRT` makes of the payload: the XML it reads back, the tag and the group.
fn through_winrt(toast: &Toast) -> (String, String, String) {
    let payload = Payload::build(toast).expect("a valid toast");
    let document = XmlDocument::new().expect("the XML document");
    document
        .LoadXml(&HSTRING::from(&payload.xml))
        .expect("the XML loads");
    let notification = ToastNotification::CreateToastNotification(&document).expect("a toast");
    notification
        .SetGroup(&HSTRING::from(GROUP))
        .expect("the group");
    notification
        .SetTag(&HSTRING::from(payload.tag.as_deref().unwrap_or_default()))
        .expect("the tag");
    let read = |text: windows::core::Result<HSTRING>| text.expect("a value").to_string();
    let content = notification.Content().expect("the content");
    (
        read(content.GetXml()),
        read(notification.Tag()),
        read(notification.Group()),
    )
}

#[test]
#[ignore = "e2e"]
fn the_xml_of_a_ping_and_of_an_urgent_toast_loads_into_the_winrt_document() {
    for (level, sound) in [
        (Level::Ping, true),
        (Level::Urgent, true),
        (Level::Ping, false),
        (Level::Urgent, false),
    ] {
        let text = format!(
            "Build <b>& \"x\" '</text><actions><action content=\"evil\"/></actions>' {level:?}"
        );
        let toast = Toast::new("Rotp: lead", text)
            .level(level)
            .url("http://rotp.localhost:47615/")
            .tag(TAG)
            .sound(sound);
        let (xml, tag, group) = through_winrt(&toast);
        assert_eq!(xml.contains("<audio silent=\"true\"/>"), !sound);
        assert_eq!((tag.as_str(), group.as_str()), (TAG, "argus"));
        assert!(
            xml.contains("<text>Build &lt;b&gt;&amp; \"x\" '&lt;/text&gt;&lt;actions&gt;"),
            "the injection is still text after the WinRT parser: {xml}"
        );
        let buttons = xml.matches("<action ").count();
        let expected = if level == Level::Urgent { 2 } else { 0 };
        assert_eq!(buttons, expected, "only the real buttons");
    }
}
