//! Plain text from a line of a program that thinks it talks to a terminal: colour and cursor
//! sequences and stray control characters are removed, so the status line shows words.

use std::sync::LazyLock;

use regex::Regex;

static ANSI: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\x1b(?:\][^\x07\x1b]*(?:\x07|\x1b\\)?|\[[0-?]*[ -/]*[@-~]|[ -/]*[0-~])").unwrap()
});

static CONTROL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\x00-\x08\x0b-\x1f\x7f]").unwrap());

pub fn plain_text(text: &str) -> String {
    CONTROL
        .replace_all(&ANSI.replace_all(text, ""), "")
        .into_owned()
}
