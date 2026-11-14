//! The secret value type and its one safe view.

use std::fmt;

/// A secret string. `Debug` and `Display` print a redaction; only `expose` gives the text.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretText(String);

impl SecretText {
    /// Wraps a value.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// The text itself. Call it only where the key is handed to its one consumer.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretText(<redacted>)")
    }
}

impl fmt::Display for SecretText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

/// The shortest key that shows a hint; a shorter one shows nothing.
const HINT_MIN_CHARS: usize = 8;
/// How many trailing characters a hint shows.
const HINT_CHARS: usize = 4;

/// The last 4 characters of a key, or an empty text when the key is too short to show any.
#[must_use]
pub fn hint_of(key: &str) -> String {
    let count = key.chars().count();
    if count < HINT_MIN_CHARS {
        return String::new();
    }
    key.chars().skip(count - HINT_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // test-tag: critical
    #[test]
    fn debug_and_display_never_show_the_value() {
        let secret = SecretText::new("sk-ant-very-secret");
        assert_eq!(format!("{secret:?}"), "SecretText(<redacted>)");
        assert_eq!(format!("{secret}"), "<redacted>");
        assert_eq!(
            format!("{:?}", Some(secret)),
            "Some(SecretText(<redacted>))"
        );
    }

    #[test]
    fn the_hint_is_the_last_four_characters() {
        assert_eq!(hint_of("sk-ant-api03-abcd"), "abcd");
    }

    #[test]
    fn a_short_or_empty_key_gives_no_hint_and_never_panics() {
        assert_eq!(hint_of("").len(), 0);
        assert_eq!(hint_of("abc").len(), 0);
        assert_eq!(hint_of("abcdefg").len(), 0);
    }

    #[test]
    fn a_hint_counts_characters_not_bytes() {
        assert_eq!(hint_of("cle-secrete-éèàù"), "éèàù");
    }
}
