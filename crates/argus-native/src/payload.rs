//! The toast payload: the `ToastGeneric` XML, the click argument and the replace tag, checked.

use crate::error::NotifyError;
use crate::text::{TEXT_MAX, TITLE_MAX, plain, xml_escape};
use crate::toast::{Level, Toast};

/// The toast group every Argus toast is shown under.
pub const GROUP: &str = "argus";
const URL_MAX: usize = 300;
const TAG_MAX: usize = 64;

/// Everything the platform needs to show one toast.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payload {
    /// The toast XML.
    pub xml: String,
    /// The replace tag, when the caller gave one.
    pub tag: Option<String>,
}

impl Payload {
    /// Checks a toast and builds its payload.
    ///
    /// # Errors
    ///
    /// `Empty` when the title and the text are both empty after cleaning, `InvalidUrl` when the
    /// click address is not a plain `http` or `https` address, `InvalidTag` when the tag is empty
    /// or longer than 64 characters.
    pub fn build(toast: &Toast) -> Result<Self, NotifyError> {
        let lines = [plain(&toast.title, TITLE_MAX), plain(&toast.text, TEXT_MAX)];
        if lines.iter().all(String::is_empty) {
            return Err(NotifyError::Empty);
        }
        let link = toast.url.as_deref().map(checked_url).transpose()?;
        let tag = toast.tag.as_deref().map(checked_tag).transpose()?;
        let xml = toast_xml(&lines, toast.level, link.as_deref(), toast.sound);
        Ok(Self { xml, tag })
    }
}

fn checked_url(url: &str) -> Result<String, NotifyError> {
    let lower = url.to_ascii_lowercase();
    let rest = lower
        .strip_prefix("http://")
        .or_else(|| lower.strip_prefix("https://"))
        .ok_or_else(|| NotifyError::InvalidUrl("only http and https open from a toast".into()))?;
    if rest.is_empty() || url.chars().count() > URL_MAX {
        return Err(NotifyError::InvalidUrl("empty or longer than 300".into()));
    }
    if url
        .chars()
        .any(|c| c.is_whitespace() || c.is_control() || matches!(c, '"' | '<' | '>'))
    {
        return Err(NotifyError::InvalidUrl(
            "spaces, quotes and brackets are not allowed".into(),
        ));
    }
    Ok(xml_escape(url))
}

fn checked_tag(tag: &str) -> Result<String, NotifyError> {
    if tag.is_empty() || tag.chars().count() > TAG_MAX {
        return Err(NotifyError::InvalidTag("1 to 64 characters".into()));
    }
    Ok(tag.to_owned())
}

fn toast_xml(lines: &[String; 2], level: Level, link: Option<&str>, sound: bool) -> String {
    let urgent = level == Level::Urgent;
    let launch = link.map_or_else(String::new, |link| {
        format!(" launch=\"{link}\" activationType=\"protocol\"")
    });
    let scenario = if urgent { " scenario=\"reminder\"" } else { "" };
    let actions = if urgent { actions(link) } else { String::new() };
    let audio = if sound {
        ""
    } else {
        "<audio silent=\"true\"/>"
    };
    format!(
        "<toast{launch}{scenario}>{}{actions}{audio}</toast>",
        visual(lines)
    )
}

fn visual(lines: &[String; 2]) -> String {
    let mut texts = String::new();
    for line in lines.iter().filter(|line| !line.is_empty()) {
        texts.push_str("<text>");
        texts.push_str(&xml_escape(line));
        texts.push_str("</text>");
    }
    format!("<visual><binding template=\"ToastGeneric\">{texts}</binding></visual>")
}

fn actions(link: Option<&str>) -> String {
    let open = link.map_or_else(String::new, |link| {
        format!("<action content=\"Open\" arguments=\"{link}\" activationType=\"protocol\"/>")
    });
    format!(
        "<actions>{open}<action content=\"Dismiss\" arguments=\"dismiss\" activationType=\"system\"/></actions>"
    )
}
