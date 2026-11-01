//! Parsers for what git prints.

use crate::paths::clean_path;

/// One entry of `git worktree list --porcelain`. The first entry is the main checkout.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListedWorktree {
    pub path: String,
    pub head: Option<String>,
    pub branch: Option<String>,
    pub detached: bool,
    pub bare: bool,
    pub locked: Option<String>,
    pub prunable: Option<String>,
}

fn mark_or(value: &str, bare: &str) -> String {
    if value.is_empty() { bare } else { value }.to_string()
}

fn branch_name(value: &str) -> String {
    value
        .strip_prefix("refs/heads/")
        .unwrap_or(value)
        .to_string()
}

fn apply_line(current: &mut ListedWorktree, key: &str, value: &str) {
    match key {
        "HEAD" => current.head = Some(value.to_string()),
        "branch" => current.branch = Some(branch_name(value)),
        "detached" => current.detached = true,
        "bare" => current.bare = true,
        "locked" => current.locked = Some(mark_or(value, "locked")),
        "prunable" => current.prunable = Some(mark_or(value, "prunable")),
        _ => {}
    }
}

/// `git worktree list --porcelain` as a list.
pub fn parse_worktree_list(text: &str) -> Vec<ListedWorktree> {
    let mut out: Vec<ListedWorktree> = Vec::new();
    let mut open = false;
    for raw in text.lines() {
        let line = raw.trim_end();
        if line.is_empty() {
            open = false;
            continue;
        }
        let (key, value) = line.split_once(' ').unwrap_or((line, ""));
        if key == "worktree" {
            out.push(ListedWorktree {
                path: clean_path(value),
                ..ListedWorktree::default()
            });
            open = true;
        } else if let (true, Some(current)) = (open, out.last_mut()) {
            apply_line(current, key, value);
        }
    }
    out
}

/// What `git status --porcelain=v1 -b` says.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StatusInfo {
    pub dirty: u32,
    pub ahead: u32,
    pub behind: u32,
    pub upstream: Option<String>,
    pub gone: bool,
}

fn count_after(counts: &str, word: &str) -> u32 {
    let Some(at) = counts.find(word) else {
        return 0;
    };
    let digits: String = counts[at + word.len()..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().unwrap_or(0)
}

fn branch_parts(line: &str) -> (Option<String>, String) {
    let rest = line.strip_prefix("## ").unwrap_or_default();
    let (main, counts) = match rest.find(" [") {
        Some(i) if rest.ends_with(']') => (&rest[..i], rest[i + 2..rest.len() - 1].to_string()),
        _ => (rest, String::new()),
    };
    if main.contains(' ') {
        return (None, String::new());
    }
    let upstream = main
        .split_once("...")
        .map(|(_, u)| u.to_string())
        .filter(|u| !u.is_empty());
    (upstream, counts)
}

/// `git status --porcelain=v1 -b` as a dirty count, ahead and behind, and the upstream.
pub fn parse_status(text: &str) -> StatusInfo {
    let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
    let head = lines
        .iter()
        .find(|l| l.starts_with("## "))
        .copied()
        .unwrap_or_default();
    let dirty = lines.iter().filter(|l| !l.starts_with("## ")).count();
    let (upstream, counts) = branch_parts(head);
    StatusInfo {
        dirty: u32::try_from(dirty).unwrap_or(u32::MAX),
        ahead: count_after(&counts, "ahead "),
        behind: count_after(&counts, "behind "),
        upstream,
        gone: counts.contains("gone"),
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    fn info(dirty: u32, ahead: u32, behind: u32, upstream: Option<&str>, gone: bool) -> StatusInfo {
        StatusInfo {
            dirty,
            ahead,
            behind,
            upstream: upstream.map(String::from),
            gone,
        }
    }

    #[test]
    fn status_dirty_count_ahead_behind_upstream_gone() {
        assert_eq!(
            parse_status("## main...origin/main [ahead 2, behind 1]\n M a.txt\n?? b.txt\n"),
            info(2, 2, 1, Some("origin/main"), false)
        );
        assert_eq!(parse_status("## feat\n"), info(0, 0, 0, None, false));
        assert_eq!(
            parse_status("## feat...origin/feat [behind 4]\n"),
            info(0, 0, 4, Some("origin/feat"), false)
        );
        assert!(parse_status("## feat...origin/feat [gone]\n").gone);
        assert_eq!(parse_status("## HEAD (no branch)\nA  x\n").dirty, 1);
        assert_eq!(parse_status("").dirty, 0);
    }

    #[test]
    fn worktree_list_main_first_branch_detached_bare_locked_prunable() {
        let text = [
            "worktree X:/proj",
            "HEAD aaaa",
            "branch refs/heads/main",
            "",
            "worktree X:/proj-wt/a",
            "HEAD bbbb",
            "branch refs/heads/feat/a",
            "",
            "worktree X:/proj-wt/b",
            "HEAD cccc",
            "detached",
            "locked in use",
            "",
            "worktree X:/proj-wt/c",
            "HEAD dddd",
            "branch refs/heads/c",
            "prunable gitdir file points to non-existent location",
            "",
            "worktree X:/bare",
            "bare",
            "",
        ]
        .join("\n");
        let list = parse_worktree_list(&text);
        let paths: Vec<&str> = list.iter().map(|x| x.path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "X:/proj",
                "X:/proj-wt/a",
                "X:/proj-wt/b",
                "X:/proj-wt/c",
                "X:/bare"
            ]
        );
        assert_eq!(list[1].branch.as_deref(), Some("feat/a"));
        assert!(list[2].detached);
        assert_eq!(list[2].locked.as_deref(), Some("in use"));
        assert!(
            list[3]
                .prunable
                .as_deref()
                .unwrap()
                .contains("non-existent")
        );
        assert!(list[4].bare);
    }

    #[test]
    fn a_bare_lock_and_a_bare_prune_mark_still_count() {
        let list = parse_worktree_list("worktree X:/a\nlocked\nprunable\n");
        assert_eq!(
            (list[0].locked.as_deref(), list[0].prunable.as_deref()),
            (Some("locked"), Some("prunable"))
        );
    }
}
