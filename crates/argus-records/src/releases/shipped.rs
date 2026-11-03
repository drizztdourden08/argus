//! What a release ships (R20, R5): the items (`I-n` in the commit subjects) and the merged pull
//! requests (`Merge pull request #n`, a squash `(#n)`) of the commits since the previous version of
//! its channel, read with `git log` from the project's checkout. Anything git cannot answer is an
//! empty list: a release page never fails for this.

use std::path::Path;
use std::process::Command;

use serde_json::{Value, json};

/// The most commits looked at when a channel has no earlier version.
const FIRST_LOOK: &str = "--max-count=200";
const RECORD: char = '\u{1e}';
const FIELD: char = '\u{1f}';

/// A merged pull request found in a commit subject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Merged {
    /// The number GitHub gave it.
    pub number: u64,
    /// The commit that merged it.
    pub commit: String,
}

/// The items and pull requests of a range.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Shipped {
    /// `I-n`, once each, lowest number first.
    pub items: Vec<String>,
    /// Merged pull requests, oldest number first.
    pub prs: Vec<Merged>,
}

impl Shipped {
    /// The `items` and `prs` fields of a version.
    #[must_use]
    pub fn json(&self) -> (Value, Value) {
        let prs: Vec<Value> = self
            .prs
            .iter()
            .map(|m| json!({ "number": m.number, "commit": m.commit }))
            .collect();
        (json!(self.items), json!(prs))
    }
}

fn digits_after(text: &str) -> Option<u64> {
    let digits: String = text.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

/// Every `I-n` in a text whose `I` does not follow a letter or digit.
fn item_numbers(subject: &str) -> Vec<u64> {
    let mut found = Vec::new();
    for (at, _) in subject.match_indices("I-") {
        let fresh = subject[..at]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_ascii_alphanumeric());
        if let Some(n) = digits_after(&subject[at + 2..]).filter(|_| fresh) {
            found.push(n);
        }
    }
    found
}

/// The pull request a subject merges: `Merge pull request #12 from ...` or `... (#12)`.
fn merged_number(subject: &str) -> Option<u64> {
    if let Some(rest) = subject.strip_prefix("Merge pull request #") {
        return digits_after(rest);
    }
    let tail = subject.trim_end().strip_suffix(')')?;
    digits_after(tail.rsplit_once("(#")?.1)
}

/// What the `git log --format=%H<US>%s<RS>` text names.
#[must_use]
pub fn parse_log(log: &str) -> Shipped {
    let mut numbers: Vec<u64> = Vec::new();
    let mut prs: Vec<Merged> = Vec::new();
    for record in log.split(RECORD) {
        let Some((commit, subject)) = record.trim().split_once(FIELD) else {
            continue;
        };
        numbers.extend(item_numbers(subject));
        if let Some(number) = merged_number(subject) {
            prs.push(Merged {
                number,
                commit: commit.to_owned(),
            });
        }
    }
    numbers.sort_unstable();
    numbers.dedup();
    prs.sort_by_key(|m| m.number);
    prs.dedup_by_key(|m| m.number);
    Shipped {
        items: numbers.into_iter().map(|n| format!("I-{n}")).collect(),
        prs,
    }
}

fn is_commit(text: &str) -> bool {
    (4..=64).contains(&text.len()) && text.chars().all(|c| c.is_ascii_hexdigit())
}

fn git_log(repo: &Path, range: &[String]) -> Option<String> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repo)
        .args(["log", "--format=%H%x1f%s%x1e"])
        .args(range);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    let out = command.output().ok().filter(|o| o.status.success())?;
    String::from_utf8(out.stdout).ok()
}

/// The items and pull requests in `previous..commit` of the checkout `repo` (the last 200 commits
/// when there is no previous one). `None` when a commit is not a hex id or git cannot answer.
#[must_use]
pub fn of_range(repo: &Path, previous: Option<&str>, commit: &str) -> Option<Shipped> {
    if !is_commit(commit) || previous.is_some_and(|p| !is_commit(p)) {
        return None;
    }
    let range = match previous {
        Some(p) => vec![format!("{p}..{commit}")],
        None => vec![FIRST_LOOK.to_owned(), commit.to_owned()],
    };
    git_log(repo, &range).map(|log| parse_log(&log))
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    fn log(lines: &[(&str, &str)]) -> String {
        lines
            .iter()
            .map(|(sha, subject)| format!("{sha}{FIELD}{subject}{RECORD}\n"))
            .collect::<Vec<_>>()
            .concat()
    }

    #[test]
    fn items_are_the_i_tokens_of_the_subjects_once_each_lowest_first() {
        let shipped = parse_log(&log(&[
            ("a1", "Agents are linked records (I-85 P2)"),
            ("b2", "Fix the pad, I-9 and I-85 again"),
            ("c3", "HAI-4 is not an item and neither is I-"),
            ("d4", "Tessera 0.95.0"),
        ]));
        assert_eq!(shipped.items, ["I-9", "I-85"]);
        assert_eq!(shipped.prs.len(), 0);
    }

    #[test]
    fn merged_prs_come_from_merge_commits_and_squash_suffixes() {
        let shipped = parse_log(&log(&[
            ("a1", "Merge pull request #12 from sync/2026-11-03"),
            ("b2", "Lockfile from the registry (#7)"),
            ("c3", "A subject with (#3) inside and more"),
            ("d4", "Merge pull request #12 from again"),
        ]));
        let numbers: Vec<u64> = shipped.prs.iter().map(|m| m.number).collect();
        assert_eq!(numbers, [7, 12]);
        assert_eq!(shipped.prs[1].commit, "a1");
        let (items, prs) = shipped.json();
        assert_eq!(items, json!([]));
        assert_eq!(prs[0], json!({"number": 7, "commit": "b2"}));
    }

    #[test]
    fn a_commit_that_is_not_hex_is_never_handed_to_git() {
        let nowhere = Path::new("Z:/does/not/exist");
        assert_eq!(of_range(nowhere, None, "--upload-pack=x"), None);
        assert_eq!(of_range(nowhere, Some("main"), "abcd1234"), None);
        assert_eq!(of_range(nowhere, Some("abcd1234"), "ef567890"), None);
    }
}
