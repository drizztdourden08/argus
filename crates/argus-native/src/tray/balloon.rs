//! The balloon tip: a notification the server could not show as a toast leaves
//! `tray-notify.json` (`{"title","text"}`, ASCII with `\u` escapes) and the tray shows it.

use std::fmt::Write;
use std::fs;
use std::io;

use argus_core::fs::write_atomic;
use serde_json::Value;

use super::paths::TrayFiles;
use crate::text::plain;

/// The longest balloon title Windows shows, in characters.
pub const TITLE_MAX: usize = 63;
/// The longest balloon text Windows shows, in characters.
pub const TEXT_MAX: usize = 255;

/// One balloon tip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Balloon {
    /// The bold first line.
    pub title: String,
    /// The text under it.
    pub text: String,
}

impl Balloon {
    /// A balloon with one-line texts cut to what Windows shows.
    #[must_use]
    pub fn new(title: &str, text: &str) -> Self {
        Self {
            title: plain(title, TITLE_MAX),
            text: plain(text, TEXT_MAX),
        }
    }

    /// The file content: ASCII only, every other character as a `\u` escape.
    #[must_use]
    pub fn to_json(&self) -> String {
        format!(
            "{{\"title\":{},\"text\":{}}}\n",
            ascii_string(&self.title),
            ascii_string(&self.text)
        )
    }
}

fn ascii_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            ' '..='~' => out.push(c),
            other => {
                let mut units = [0u16; 2];
                for unit in other.encode_utf16(&mut units) {
                    let _written = write!(out, "\\u{unit:04x}");
                }
            }
        }
    }
    out.push('"');
    out
}

/// Leaves a balloon request for the tray.
///
/// # Errors
///
/// The home cannot be created or the file cannot be written.
pub fn write(files: &TrayFiles, balloon: &Balloon) -> io::Result<()> {
    write_atomic(&files.balloon_file(), balloon.to_json())
}

/// Takes the pending request: reads it and deletes the file, whether or not it parsed.
#[must_use]
pub fn take(files: &TrayFiles) -> Option<Balloon> {
    let text = fs::read_to_string(files.balloon_file()).ok()?;
    let _taken = fs::remove_file(files.balloon_file());
    let json: Value = serde_json::from_str(&text).ok()?;
    let field = |name: &str| json.get(name).and_then(Value::as_str).unwrap_or_default();
    Some(Balloon::new(field("title"), field("text")))
}

#[cfg(test)]
mod tests {
    use super::{Balloon, TEXT_MAX, TITLE_MAX, take, write};
    use crate::tray::paths::TrayFiles;

    #[test]
    fn the_file_is_ascii_and_round_trips_other_characters() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        let balloon = Balloon::new("Caf\u{e9} \"quoted\"", "na\u{ef}ve \\ \u{1F680} done");
        write(&files, &balloon).unwrap();
        let raw = std::fs::read_to_string(files.balloon_file()).unwrap();
        assert!(raw.is_ascii());
        assert!(raw.contains("\\u00e9") && raw.contains("\\ud83d\\ude80"));
        assert_eq!(take(&files).unwrap(), balloon);
    }

    #[test]
    fn taking_deletes_the_file_even_when_it_is_broken() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        assert!(take(&files).is_none());
        std::fs::write(files.balloon_file(), "{broken").unwrap();
        assert!(take(&files).is_none());
        assert!(!files.balloon_file().exists());
    }

    #[test]
    fn long_texts_are_cut_to_what_windows_shows() {
        let balloon = Balloon::new(&"t".repeat(200), &"x".repeat(900));
        assert_eq!(balloon.title.chars().count(), TITLE_MAX);
        assert_eq!(balloon.text.chars().count(), TEXT_MAX);
    }
}
