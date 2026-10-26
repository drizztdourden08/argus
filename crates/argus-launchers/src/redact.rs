//! Secret scrubbing for run logs (DESIGN.md section 8). Known secret values (the dashboard
//! token, secret-looking env values of a definition) are replaced verbatim; the patterns
//! catch the common shapes of tokens nobody told us about.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use regex::Regex;

/// A value shorter than this would blank out ordinary words, so it is not a secret.
const MIN_SECRET: usize = 4;
const REDACTED: &str = "[redacted]";

static PATTERNS: LazyLock<[(Regex, &'static str); 3]> = LazyLock::new(|| {
    [
        (
            Regex::new(r"(?i)(authorization:\s*bearer\s+)\S+").unwrap(),
            "${1}[redacted]",
        ),
        (
            Regex::new(r"(?i)((?:api[_-]?key|token|secret|password)\s*[=:]\s*)\S+").unwrap(),
            "${1}[redacted]",
        ),
        (
            Regex::new(
                r"\b(sk-[A-Za-z0-9_-]{20,}|gh[pousr]_[A-Za-z0-9]{30,}|xox[abp]-[A-Za-z0-9-]{10,})\b",
            )
            .unwrap(),
            REDACTED,
        ),
    ]
});

static SECRET_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("(?i)token|secret|pass|key|auth").unwrap());

/// The env values of a definition that look like secrets, for [`Redactor::new`].
pub fn secret_env_values(env: &BTreeMap<String, String>) -> Vec<String> {
    env.iter()
        .filter(|(key, _)| SECRET_KEY.is_match(key))
        .map(|(_, value)| value.clone())
        .collect()
}

#[derive(Clone, Debug, Default)]
pub struct Redactor {
    secrets: Vec<String>,
}

impl Redactor {
    /// `extra` are the known secrets; short and empty ones are ignored.
    pub fn new<S: AsRef<str>>(extra: &[S]) -> Self {
        let mut secrets: Vec<String> = extra
            .iter()
            .map(|s| s.as_ref().to_owned())
            .filter(|s| s.chars().count() >= MIN_SECRET)
            .collect();
        secrets.sort_by_key(|s| std::cmp::Reverse(s.len()));
        secrets.dedup();
        Self { secrets }
    }

    pub fn redact(&self, text: &str) -> String {
        let mut out = text.to_owned();
        for secret in &self.secrets {
            out = out.replace(secret.as_str(), REDACTED);
        }
        for (pattern, replacement) in PATTERNS.iter() {
            out = pattern.replace_all(&out, *replacement).into_owned();
        }
        out
    }
}
