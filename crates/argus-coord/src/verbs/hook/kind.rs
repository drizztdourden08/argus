//! The kind of a hook-made agent record, read from the words of its task: `test`, `chore`,
//! `research`, else `build`. A word counts only as a whole word, in any case.

const KINDS: [(&str, &[&str]); 3] = [
    (
        "test",
        &[
            "test",
            "tests",
            "testing",
            "lint",
            "typecheck",
            "verify",
            "suite",
            "qa",
        ],
    ),
    (
        "chore",
        &[
            "chore",
            "chores",
            "rename",
            "format",
            "cleanup",
            "clean up",
            "bump",
            "upgrade",
            "housekeeping",
        ],
    ),
    (
        "research",
        &[
            "research",
            "investigate",
            "explore",
            "find",
            "search",
            "audit",
            "review",
            "analyse",
            "analyze",
            "survey",
            "read",
        ],
    ),
];

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Whether `word` stands in `text` (lower case) with a word boundary on each side.
fn has_word(text: &str, word: &str) -> bool {
    let bytes = text.as_bytes();
    text.match_indices(word).any(|(at, _)| {
        let end = at + word.len();
        let open = at == 0 || !is_word_byte(bytes[at - 1]);
        let close = end == bytes.len() || !is_word_byte(bytes[end]);
        open && close
    })
}

/// The kind of a task text.
pub fn kind_of(text: &str) -> &'static str {
    let lower = text.to_ascii_lowercase();
    KINDS
        .iter()
        .find(|(_, words)| words.iter().any(|w| has_word(&lower, w)))
        .map_or("build", |(kind, _)| kind)
}
