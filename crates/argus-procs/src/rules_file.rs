//! The rules file `process-rules.json`: its text and its parsing.

use crate::defaults::default_rules;
use crate::normalize::normalize_rule;
use crate::rule::Rule;
use serde_json::{Value, json};
use std::collections::HashSet;

pub const RULES_FILE: &str = "process-rules.json";

const COMMENT_HEAD: [&str; 5] = [
    "Process rules of the Argus process monitor (DESIGN.md 13.10, shared/skills/coord/processes.md). Edit them with `coord proc rule set|rm`, not by hand.",
    "Fields: match.name / match.cmd / match.path: regexes (case-insensitive, forward slashes) on the executable name, the command line, the executable path; all given keys must match, an empty match takes everything.",
    "ignore: regexes on the command line that exempt a process from the rule (it goes on to the next rule). The first rule that takes a process wins.",
    "minMemMB: listed only above this. onlyFlagged: listed only when over a limit. maxMemMB: per process. maxGroupMemMB: all processes of the rule together.",
    "maxAgeMin: running longer is \"old\". idleMin: CPU under 0.5% of one core this long is \"stale\". orphan: its parent is gone. hung: its window does not respond. killable: the kill buttons and `coord proc kill` may end it.",
];
const COMMENT_TAIL: &str = "Always protected, whatever a rule says: system processes (csrss, winlogon, services, lsass, svchost, explorer, dwm, ...), anything under the Windows folder, Claude Desktop and Claude Code sessions, the dashboard server and its parents, the tray, the wake listeners and the monitor's own sampler.";

fn comment_lines() -> Vec<String> {
    let mut lines: Vec<String> = COMMENT_HEAD.iter().map(ToString::to_string).collect();
    lines.extend(
        default_rules()
            .iter()
            .map(|r| format!("{}: {}", r.name, r.note.as_deref().unwrap_or(""))),
    );
    lines.push(COMMENT_TAIL.to_string());
    lines
}

/// The text of the rules file for these rules.
#[must_use]
pub fn rules_file_text(rules: &[Rule]) -> String {
    let doc = json!({"_comment": comment_lines(), "version": 1, "rules": rules});
    let mut text = serde_json::to_string_pretty(&doc).unwrap_or_default();
    text.push('\n');
    text
}

fn checked_rules(list: &[Value]) -> Result<Vec<Rule>, String> {
    let mut seen = HashSet::new();
    list.iter()
        .map(|item| {
            let rule = normalize_rule(item).map_err(|e| e.message().to_string())?;
            if seen.insert(rule.name.clone()) {
                Ok(rule)
            } else {
                Err(format!("rule \"{}\" appears twice", rule.name))
            }
        })
        .collect()
}

/// The rules of the file's text, or the reason it cannot be used (the caller keeps what it had).
///
/// # Errors
/// The text is not JSON, has no rules array, or holds a bad or repeated rule.
pub fn parse_rules_file(text: &str) -> Result<Vec<Rule>, String> {
    let data: Value =
        serde_json::from_str(text).map_err(|e| format!("{RULES_FILE} is not valid JSON: {e}"))?;
    let list = data.as_array().or_else(|| data.get("rules")?.as_array());
    let list = list.ok_or_else(|| format!("{RULES_FILE} has no \"rules\" array"))?;
    checked_rules(list).map_err(|e| format!("{RULES_FILE}: {e}"))
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_text_carries_a_comment_and_every_default() {
        let text = rules_file_text(&default_rules());
        let back: Value = serde_json::from_str(&text).unwrap();
        let comment = back["_comment"].as_array().unwrap();
        assert!(
            comment
                .iter()
                .any(|l| l.as_str().unwrap().starts_with("catch-all:"))
        );
        assert_eq!(parse_rules_file(&text).unwrap(), default_rules());
    }

    #[test]
    fn a_bad_file_says_why() {
        assert!(
            parse_rules_file("{ not json")
                .unwrap_err()
                .contains("not valid JSON")
        );
        assert!(
            parse_rules_file("{}")
                .unwrap_err()
                .contains("no \"rules\" array")
        );
        let twice = parse_rules_file(r#"{"rules":[{"name":"a"},{"name":"a"}]}"#);
        assert!(twice.unwrap_err().contains("appears twice"));
    }
}
