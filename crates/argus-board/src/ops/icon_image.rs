//! The image kinds of a board icon: a png or svg data URL, its decoded bytes and the safety checks
//! an svg must pass before it is kept. The server decodes the same value to serve it.

use std::sync::LazyLock;

use regex::Regex;

const PNG_PREFIX: &str = "data:image/png;base64,";
const SVG_PREFIX: &str = "data:image/svg+xml;base64,";

/// The most characters a whole icon data URL may take.
const ICON_MAX: usize = 64 * 1024;

/// The most bytes a decoded svg may take.
const SVG_MAX: usize = 32 * 1024;

const SVG_BANNED: [&str; 3] = ["<script", "<foreignobject", "javascript:"];

static HANDLER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?i)[\s/'"]on[a-z]+\s*="#).expect("handler pattern"));
static EXTERNAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?i)href\s*=\s*["']?\s*(?:https?:|//)"#).expect("href pattern"));

fn sextet(b: u8) -> Option<u32> {
    match b {
        b'A'..=b'Z' => Some(u32::from(b - b'A')),
        b'a'..=b'z' => Some(u32::from(b - b'a') + 26),
        b'0'..=b'9' => Some(u32::from(b - b'0') + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn decode_group(group: &[u8]) -> Option<Vec<u8>> {
    let pad = group.iter().filter(|b| **b == b'=').count();
    if pad > 2 || group[..4 - pad].contains(&b'=') {
        return None;
    }
    let mut bits = 0_u32;
    for b in &group[..4 - pad] {
        bits = (bits << 6) | sextet(*b)?;
    }
    bits <<= 6 * pad;
    let bytes = bits.to_be_bytes();
    Some(bytes[1..4 - pad].to_vec())
}

fn decode_base64(body: &str) -> Option<Vec<u8>> {
    let raw = body.as_bytes();
    if raw.is_empty() || raw.len() % 4 != 0 {
        return None;
    }
    let last = raw.len() / 4 - 1;
    let mut out = Vec::with_capacity(raw.len() / 4 * 3);
    for (n, group) in raw.chunks(4).enumerate() {
        if n != last && group.contains(&b'=') {
            return None;
        }
        out.extend(decode_group(group)?);
    }
    Some(out)
}

fn check_svg(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > SVG_MAX {
        return Err(format!(
            "icon svg is {} bytes, the limit is {SVG_MAX}.",
            bytes.len()
        ));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| "icon svg is not text.".to_owned())?;
    let lower = text.to_ascii_lowercase();
    if let Some(bad) = SVG_BANNED.iter().find(|b| lower.contains(**b)) {
        return Err(format!("icon svg may not contain {bad}."));
    }
    if HANDLER.is_match(text) {
        return Err("icon svg may not contain an event handler.".to_owned());
    }
    if EXTERNAL.is_match(text) {
        return Err("icon svg may not link to an external address.".to_owned());
    }
    Ok(())
}

/// The content type and bytes of an image icon value; `None` for anything else.
#[must_use]
pub fn icon_bytes(value: &str) -> Option<(&'static str, Vec<u8>)> {
    let (kind, body) = match value.strip_prefix(PNG_PREFIX) {
        Some(b) => ("image/png", b),
        None => ("image/svg+xml", value.strip_prefix(SVG_PREFIX)?),
    };
    Some((kind, decode_base64(body)?))
}

/// Whether a value is a png or svg data URL, checked or not.
#[must_use]
pub fn is_image_icon(value: &str) -> bool {
    value.starts_with(PNG_PREFIX) || value.starts_with(SVG_PREFIX)
}

/// Checks an image icon.
///
/// # Errors
/// A plain message when the URL is too long, not base64, or an unsafe svg.
pub(crate) fn check_image(value: &str) -> Result<(), String> {
    if value.len() > ICON_MAX {
        return Err(format!(
            "icon image is {} chars, the limit is {ICON_MAX}. Use a smaller image.",
            value.len()
        ));
    }
    let (kind, bytes) = icon_bytes(value).ok_or("icon image is not valid base64.")?;
    if kind == "image/svg+xml" {
        check_svg(&bytes)?;
    }
    Ok(())
}
