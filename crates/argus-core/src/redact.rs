//! Secrets out of logs: literal values (the token) and the patterns of DESIGN 8.

use std::sync::LazyLock;

use regex::Regex;

static BEARER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(authorization:\s*bearer\s+)\S+").expect("valid pattern"));
static ASSIGN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)((?:api[_-]?key|token|secret|password)\s*[=:]\s*)\S+").expect("valid pattern")
});
static KEYS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(sk-[A-Za-z0-9_-]{20,}|gh[pousr]_[A-Za-z0-9]{30,}|xox[abp]-[A-Za-z0-9-]{10,})\b")
        .expect("valid pattern")
});

const MARK: &str = "[redacted]";

/// Replaces secrets in a text.
#[derive(Debug, Clone, Default)]
pub struct Redactor {
    literals: Vec<String>,
}

impl Redactor {
    /// A redactor that also hides each literal of 8 characters or more.
    #[must_use]
    pub fn new(extra: &[String]) -> Self {
        Redactor {
            literals: extra.iter().filter(|s| s.len() >= 8).cloned().collect(),
        }
    }

    /// The text with every secret replaced by `[redacted]`.
    #[must_use]
    pub fn redact(&self, text: &str) -> String {
        let mut out = text.to_owned();
        for lit in &self.literals {
            out = out.replace(lit.as_str(), MARK);
        }
        let out = BEARER.replace_all(&out, "${1}[redacted]");
        let out = ASSIGN.replace_all(&out, "${1}[redacted]");
        KEYS.replace_all(&out, MARK).into_owned()
    }
}
