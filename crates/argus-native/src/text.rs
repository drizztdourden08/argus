//! Plain text for toasts: the cleaning every person-written string goes through, and XML escaping.
// test-tag: critical

/// The longest toast title, in characters.
pub const TITLE_MAX: usize = 60;
/// The longest toast body, in characters.
pub const TEXT_MAX: usize = 240;

const ELLIPSIS: char = '\u{2026}';

/// True for characters that never belong in a one-line notification: control characters, line and
/// paragraph separators, zero-width and bidirectional marks, and the BOM and non-characters.
fn is_hidden(c: char) -> bool {
    matches!(
        c,
        '\u{0000}'..='\u{001f}'
            | '\u{007f}'..='\u{009f}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{200b}'..='\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2066}'..='\u{2069}'
            | '\u{feff}'
            | '\u{fffe}'
            | '\u{ffff}'
    )
}

/// One line of plain text with at most `max` characters. Hidden characters become a space, runs of
/// whitespace collapse, and a cut ends in an ellipsis.
#[must_use]
pub fn plain(value: &str, max: usize) -> String {
    let spaced: String = value
        .chars()
        .map(|c| if is_hidden(c) { ' ' } else { c })
        .collect();
    let line = spaced.split_whitespace().collect::<Vec<_>>().join(" ");
    if line.chars().count() <= max {
        return line;
    }
    let mut cut: String = line.chars().take(max.saturating_sub(1)).collect();
    cut.truncate(cut.trim_end().len());
    cut.push(ELLIPSIS);
    cut
}

/// Escapes the five XML special characters, so text can sit in an element or an attribute.
#[must_use]
pub fn xml_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{TEXT_MAX, plain, xml_escape};

    #[test]
    fn hidden_characters_and_line_breaks_become_one_space() {
        assert_eq!(plain("a\r\nb\t\u{200b}c\u{202e}d   e", 100), "a b c d e");
        assert_eq!(plain("  \u{feff}x  ", 100), "x");
    }

    #[test]
    fn a_long_text_is_cut_with_an_ellipsis() {
        let cut = plain(&"x".repeat(TEXT_MAX + 50), TEXT_MAX);
        assert_eq!(cut.chars().count(), TEXT_MAX);
        assert!(cut.ends_with('\u{2026}'));
        assert_eq!(plain(&"y".repeat(TEXT_MAX), TEXT_MAX), "y".repeat(TEXT_MAX));
    }

    #[test]
    fn a_cut_never_leaves_trailing_space_before_the_ellipsis() {
        assert_eq!(plain("abc def", 5), "abc\u{2026}");
    }

    #[test]
    fn a_cut_counts_characters_not_bytes() {
        assert_eq!(plain("\u{e9}\u{e9}\u{e9}\u{e9}", 3), "\u{e9}\u{e9}\u{2026}");
    }

    #[test]
    fn the_five_xml_characters_are_escaped() {
        assert_eq!(
            xml_escape(r#"<a href="x">&'</a>"#),
            "&lt;a href=&quot;x&quot;&gt;&amp;&apos;&lt;/a&gt;"
        );
    }
}
