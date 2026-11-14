//! The child env, built by one pure function. The key lives only here and in the child.

use std::fmt;

use crate::ports::ApiKey;

/// Auth variables of the parent that must never reach the child.
const STRIPPED: [&str; 3] = [
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_AUTH_TOKEN",
    "CLAUDE_CODE_OAUTH_TOKEN",
];

/// Variables this builder sets itself, so a parent copy is dropped first.
const OWNED: [&str; 2] = ["ARGUS_HOSTED", "MCP_TOOL_TIMEOUT"];

/// Extra time on top of the approval timeout before a held MCP call gives up, in seconds.
const MARGIN_SECS: u64 = 60;

/// The env of the child. `Debug` prints the names only, never a value.
#[derive(Clone, PartialEq, Eq)]
pub struct ChildEnv(Vec<(String, String)>);

impl ChildEnv {
    /// The name and value pairs.
    #[must_use]
    pub fn pairs(&self) -> &[(String, String)] {
        &self.0
    }

    /// The value of `name` (compared without case, as Windows does).
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

impl fmt::Debug for ChildEnv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(|(name, _)| name))
            .finish()
    }
}

/// The parent env minus the three auth variables, plus the key, `ARGUS_HOSTED=1` and
/// `MCP_TOOL_TIMEOUT` (the approval timeout plus a margin, in milliseconds).
#[must_use]
pub fn build_env(
    parent: impl IntoIterator<Item = (String, String)>,
    key: &ApiKey,
    approval_timeout_min: u64,
) -> ChildEnv {
    let mut pairs: Vec<(String, String)> = parent
        .into_iter()
        .filter(|(name, _)| !is_one_of(name, &STRIPPED) && !is_one_of(name, &OWNED))
        .collect();
    pairs.push(("ANTHROPIC_API_KEY".to_owned(), key.expose().to_owned()));
    pairs.push(("ARGUS_HOSTED".to_owned(), "1".to_owned()));
    pairs.push((
        "MCP_TOOL_TIMEOUT".to_owned(),
        tool_timeout_ms(approval_timeout_min).to_string(),
    ));
    ChildEnv(pairs)
}

fn is_one_of(name: &str, names: &[&str]) -> bool {
    names
        .iter()
        .any(|candidate| name.eq_ignore_ascii_case(candidate))
}

fn tool_timeout_ms(approval_timeout_min: u64) -> u64 {
    approval_timeout_min
        .saturating_mul(60)
        .saturating_add(MARGIN_SECS)
        .saturating_mul(1000)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parent() -> Vec<(String, String)> {
        [
            ("PATH", "C:/bin"),
            ("ANTHROPIC_API_KEY", "sk-parent"),
            ("ANTHROPIC_AUTH_TOKEN", "tok-parent"),
            ("claude_code_oauth_token", "oauth-parent"),
            ("MCP_TOOL_TIMEOUT", "5"),
        ]
        .iter()
        .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
        .collect()
    }

    #[test]
    fn the_child_gets_the_key_and_not_the_parents_auth() {
        let env = build_env(parent(), &ApiKey::new("sk-child"), 30);
        assert_eq!(env.get("ANTHROPIC_API_KEY"), Some("sk-child"));
        assert_eq!(env.get("ANTHROPIC_AUTH_TOKEN"), None);
        assert_eq!(env.get("CLAUDE_CODE_OAUTH_TOKEN"), None);
        let keys = env
            .pairs()
            .iter()
            .filter(|(name, _)| name == "ANTHROPIC_API_KEY")
            .count();
        assert_eq!(keys, 1);
        assert!(
            env.pairs()
                .iter()
                .all(|(_, value)| !value.contains("parent"))
        );
    }

    #[test]
    fn the_hosted_flag_and_the_timeout_are_set_and_the_rest_kept() {
        let env = build_env(parent(), &ApiKey::new("sk-child"), 30);
        assert_eq!(env.get("ARGUS_HOSTED"), Some("1"));
        assert_eq!(env.get("MCP_TOOL_TIMEOUT"), Some("1860000"));
        assert_eq!(env.get("PATH"), Some("C:/bin"));
    }

    #[test]
    fn debug_never_prints_a_value() {
        let env = build_env(parent(), &ApiKey::new("sk-child"), 30);
        let text = format!("{env:?}");
        assert!(!text.contains("sk-child"));
        assert!(text.contains("ANTHROPIC_API_KEY"));
    }
}
