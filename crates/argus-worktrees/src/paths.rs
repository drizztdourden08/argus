//! Path forms shared by the rules, the scan and the guards.
//!
//! Every path the crate keeps is a clean string: forward slashes, no repeated or trailing slash.
//! Two paths are compared through [`path_key`], which also folds case where the file system does.

use sha1::{Digest, Sha1};
use std::fmt::Write;

/// Forward slashes, no repeated or trailing slash (a root such as `X:/` or `/` keeps its one).
/// A UNC prefix stays `//server/share`.
pub fn clean_path(value: &str) -> String {
    let s = value.trim().replace('\\', "/");
    let unc = s.starts_with("//") && s[2..].chars().next().is_some_and(|c| c != '/');
    let rest = if unc { &s[2..] } else { s.as_str() };
    let mut body = collapse_slashes(rest);
    if body.len() > 1 && body.ends_with('/') && !is_drive_root(&body) {
        body.pop();
    }
    if unc { format!("//{body}") } else { body }
}

fn collapse_slashes(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut previous_slash = false;
    for c in text.chars() {
        if c == '/' && previous_slash {
            continue;
        }
        previous_slash = c == '/';
        out.push(c);
    }
    out
}

fn is_drive(text: &str) -> bool {
    let b = text.as_bytes();
    b.len() == 2 && b[0].is_ascii_alphabetic() && b[1] == b':'
}

fn is_drive_root(text: &str) -> bool {
    text.len() == 3 && is_drive(&text[..2]) && text.ends_with('/')
}

/// The form two paths are compared in: clean, and lower case where the file system ignores case.
pub fn path_key(value: &str, case_insensitive: bool) -> String {
    let clean = clean_path(value);
    if case_insensitive {
        clean.to_lowercase()
    } else {
        clean
    }
}

/// The id of a worktree: ten hex characters of the hash of its path key.
pub fn worktree_id(value: &str, case_insensitive: bool) -> String {
    let digest = Sha1::digest(path_key(value, case_insensitive).as_bytes());
    let mut id = String::with_capacity(10);
    for byte in digest.iter().take(5) {
        let _ = write!(id, "{byte:02x}");
    }
    id
}

/// Is this text a worktree id (ten lower case hex characters)?
pub fn is_worktree_id(text: &str) -> bool {
    text.len() == 10 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Do the two paths name the same place?
pub fn same_path(a: &str, b: &str, case_insensitive: bool) -> bool {
    path_key(a, case_insensitive) == path_key(b, case_insensitive)
}

/// Is `inner` strictly below `outer`?
pub fn is_below(inner: &str, outer: &str, case_insensitive: bool) -> bool {
    let prefix = format!(
        "{}/",
        path_key(outer, case_insensitive).trim_end_matches('/')
    );
    path_key(inner, case_insensitive).starts_with(&prefix)
}

/// The last segment of a path.
pub fn base_name(value: &str) -> String {
    clean_path(value)
        .split('/')
        .rfind(|s| !s.is_empty())
        .unwrap_or_default()
        .to_string()
}

/// The folder that holds a path (a drive root or `/` is its own parent).
pub fn parent_of(value: &str) -> String {
    let s = clean_path(value);
    match s.rfind('/') {
        None => s,
        Some(0) => "/".to_string(),
        Some(i) => {
            let parent = &s[..i];
            if is_drive(parent) {
                format!("{parent}/")
            } else {
                parent.to_string()
            }
        }
    }
}

/// `dir` and `name` joined with one slash.
pub fn join_path(dir: &str, name: &str) -> String {
    if dir.ends_with('/') {
        format!("{dir}{name}")
    } else {
        format!("{dir}/{name}")
    }
}

/// Any wildcard a rule path may not have.
pub fn has_wildcard(value: &str) -> bool {
    value.contains(['*', '?', '[', '{'])
}

/// Does one folder name match a name glob (`*` any characters, `?` one character)?
pub fn name_glob_matches(glob: &str, name: &str, case_insensitive: bool) -> bool {
    let (glob, name) = if case_insensitive {
        (glob.to_lowercase(), name.to_lowercase())
    } else {
        (glob.to_string(), name.to_string())
    };
    let glob: Vec<char> = glob.chars().collect();
    let name: Vec<char> = name.chars().collect();
    glob_match(&glob, &name)
}

fn glob_match(glob: &[char], name: &[char]) -> bool {
    let (mut g, mut n) = (0, 0);
    let mut star: Option<(usize, usize)> = None;
    while n < name.len() {
        if g < glob.len() && glob[g] != '*' && (glob[g] == '?' || glob[g] == name[n]) {
            g += 1;
            n += 1;
        } else if g < glob.len() && glob[g] == '*' {
            star = Some((g, n));
            g += 1;
        } else if let Some((star_g, star_n)) = star {
            star = Some((star_g, star_n + 1));
            g = star_g + 1;
            n = star_n + 1;
        } else {
            return false;
        }
    }
    glob[g..].iter().all(|c| *c == '*')
}

/// Does a name pass the include list (when there is one) and miss the exclude list?
pub fn name_allowed(
    include: &[String],
    exclude: &[String],
    name: &str,
    case_insensitive: bool,
) -> bool {
    let hit = |globs: &[String]| {
        globs
            .iter()
            .any(|g| name_glob_matches(g, name, case_insensitive))
    };
    (include.is_empty() || hit(include)) && !hit(exclude)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_path_unifies_slashes_and_keeps_roots() {
        assert_eq!(clean_path("X:\\a\\\\b\\"), "X:/a/b");
        assert_eq!(clean_path("X:\\"), "X:/");
        assert_eq!(clean_path("/"), "/");
        assert_eq!(clean_path("//server/share/x/"), "//server/share/x");
        assert_eq!(clean_path("  /a//b  "), "/a/b");
    }

    #[test]
    fn base_name_and_parent() {
        assert_eq!(base_name("X:/a/b/"), "b");
        assert_eq!(parent_of("X:/a/b"), "X:/a");
        assert_eq!(parent_of("X:/a"), "X:/");
        assert_eq!(parent_of("X:/"), "X:/");
        assert_eq!(parent_of("/a"), "/");
        assert_eq!(join_path("X:/", "a"), "X:/a");
        assert_eq!(join_path("X:/a", "b"), "X:/a/b");
    }

    #[test]
    fn ids_are_ten_hex_characters_of_the_path_key() {
        let id = worktree_id("X:/Proj/wt", true);
        assert!(is_worktree_id(&id));
        assert_eq!(id, worktree_id("x:\\proj\\WT\\", true));
        assert_ne!(worktree_id("X:/a", false), worktree_id("X:/A", false));
        assert!(!is_worktree_id("ABCDEF0123"));
    }

    #[test]
    fn wildcards_in_a_rule_path() {
        for bad in ["X:/a/web-*", "X:/a/b?", "X:/a/[x]", "X:/a/{b,c}"] {
            assert!(has_wildcard(bad), "{bad}");
        }
        assert!(!has_wildcard("X:/a/b"));
    }

    #[test]
    fn name_filters_choose_direct_child_names() {
        let web = vec!["web-*".to_string()];
        let names = ["web-1", "WEB-2", "web-", "xweb-1", "agent"];
        let win: Vec<bool> = names
            .iter()
            .map(|n| name_allowed(&web, &[], n, true))
            .collect();
        assert_eq!(win, [true, true, true, false, false]);
        let unix: Vec<bool> = ["web-1", "WEB-2"]
            .iter()
            .map(|n| name_allowed(&web, &[], n, false))
            .collect();
        assert_eq!(unix, [true, false]);
        let both = (vec!["a?".to_string()], vec!["ab".to_string()]);
        let got: Vec<bool> = ["ab", "ac", "a", "abc"]
            .iter()
            .map(|n| name_allowed(&both.0, &both.1, n, false))
            .collect();
        assert_eq!(got, [false, true, false, false]);
        assert!(name_allowed(&[], &[], "anything", false));
        assert!(!name_allowed(&["web*/x".to_string()], &[], "web-1", false));
    }
}
