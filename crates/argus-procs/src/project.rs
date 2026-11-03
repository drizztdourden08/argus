//! Which project a process belongs to, from the roots the projects live in.

use crate::text::slashed;
use crate::types::ProcRow;

/// A project key and the folder its files live in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRoot {
    pub key: String,
    pub root: String,
}

/// A project key and every folder that is its own: the root, the worktrees, the ci root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectFolders {
    pub key: String,
    pub folders: Vec<String>,
}

/// Roots as lower case forward-slash paths, the longest first.
#[must_use]
pub fn normalize_roots(roots: &[ProjectRoot]) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = roots
        .iter()
        .filter(|r| !r.root.is_empty())
        .map(|r| {
            let root = slashed(&r.root).to_lowercase();
            (
                root.strip_suffix('/').unwrap_or(&root).to_string(),
                r.key.clone(),
            )
        })
        .collect();
    out.sort_by_key(|(root, _)| std::cmp::Reverse(root.len()));
    out
}

fn drive_folder(text: &str) -> Option<String> {
    text.match_indices("x:/").find_map(|(at, _)| {
        let before = text[..at].chars().next_back();
        let boundary = before.is_none_or(|c| c.is_whitespace() || matches!(c, '"' | '\'' | '='));
        let name: String = text[at + 3..]
            .chars()
            .take_while(|c| !c.is_whitespace() && !matches!(c, '/' | '"' | '\''))
            .collect();
        (boundary && !name.is_empty()).then_some(name)
    })
}

/// The key whose root is in the command line or path, else the folder after `X:/`, else `None`.
#[must_use]
pub fn project_of(proc: &ProcRow, roots: &[(String, String)]) -> Option<String> {
    let path = proc.path.as_deref().unwrap_or("");
    let text = format!("{} {}", slashed(&proc.cmd), slashed(path)).to_lowercase();
    let by_root = roots.iter().find(|(root, _)| {
        text.contains(&format!("{root}/"))
            || text.contains(&format!("{root} "))
            || text.ends_with(root.as_str())
    });
    by_root
        .map(|(_, key)| key.clone())
        .or_else(|| drive_folder(&text))
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    fn row(cmd: &str, path: Option<&str>) -> ProcRow {
        ProcRow {
            pid: 1,
            ppid: 0,
            name: "node.exe".into(),
            cmd: cmd.into(),
            path: path.map(Into::into),
            mem_bytes: 0,
            start_ms: 0,
            cpu_ms: 0,
            responding: None,
            cwd: None,
        }
    }

    #[test]
    fn roots_win_then_the_drive_folder() {
        let roots = normalize_roots(&[ProjectRoot {
            key: "alpha".into(),
            root: "X:\\alpha".into(),
        }]);
        assert_eq!(
            project_of(&row("node x:/alpha/node_modules/vite.js", None), &roots).as_deref(),
            Some("alpha")
        );
        assert_eq!(
            project_of(&row("node X:\\beta\\run.js", None), &roots).as_deref(),
            Some("beta")
        );
        assert_eq!(project_of(&row("node c:/other.js", None), &roots), None);
    }
}
