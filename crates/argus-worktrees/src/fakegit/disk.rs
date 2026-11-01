//! The working tree on disk against the tracked files: what `git status` and `git ls-files --others` would say.

use crate::fs::meta_is_link;
use crate::paths::name_glob_matches;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

struct Pattern {
    glob: String,
    negate: bool,
    dir_only: bool,
    anchored: bool,
}

/// The ignore patterns that apply to one worktree: `info/exclude` of the repository and the root `.gitignore`.
pub(super) struct Patterns(Vec<Pattern>);

fn parse_line(line: &str) -> Option<Pattern> {
    let text = line.trim();
    if text.is_empty() || text.starts_with('#') {
        return None;
    }
    let (negate, text) = text
        .strip_prefix('!')
        .map_or((false, text), |rest| (true, rest));
    let (dir_only, text) = text
        .strip_suffix('/')
        .map_or((false, text), |rest| (true, rest));
    let anchored = text.contains('/');
    Some(Pattern {
        glob: text.trim_start_matches('/').to_string(),
        negate,
        dir_only,
        anchored,
    })
}

impl Patterns {
    pub fn read(repo: &str, worktree: &str) -> Self {
        let mut all = Vec::new();
        for file in [
            format!("{repo}/.git/info/exclude"),
            format!("{worktree}/.gitignore"),
        ] {
            let text = fs::read_to_string(file).unwrap_or_default();
            all.extend(text.lines().filter_map(parse_line));
        }
        Self(all)
    }

    fn matches(&self, rel: &str, is_dir: bool) -> bool {
        let name = rel.rsplit('/').next().unwrap_or(rel);
        let mut hit = false;
        for pattern in &self.0 {
            if pattern.dir_only && !is_dir {
                continue;
            }
            let subject = if pattern.anchored { rel } else { name };
            if name_glob_matches(&pattern.glob, subject, false) {
                hit = !pattern.negate;
            }
        }
        hit
    }

    /// Is the path ignored, itself or through a folder above it?
    pub fn ignored(&self, rel: &str, is_dir: bool) -> bool {
        let parts: Vec<&str> = rel.split('/').collect();
        (0..parts.len()).any(|i| {
            let last = i + 1 == parts.len();
            self.matches(&parts[..=i].join("/"), !last || is_dir)
        })
    }
}

/// What the disk holds that the tracked files do not say.
#[derive(Default)]
pub(super) struct Dirt {
    /// The lines of `git status --porcelain`.
    pub lines: Vec<String>,
    /// The paths `git ls-files --others --exclude-standard` lists.
    pub others: Vec<String>,
    /// The tracked paths whose text changed or that are gone.
    pub changed: Vec<String>,
}

struct Walk<'a> {
    patterns: &'a Patterns,
    tracked: &'a BTreeMap<String, String>,
}

fn join(rel: &str, name: &str) -> String {
    if rel.is_empty() {
        name.to_string()
    } else {
        format!("{rel}/{name}")
    }
}

impl Walk<'_> {
    fn dir(&self, abs: &Path, rel: &str, out: &mut Dirt) {
        let Ok(read) = fs::read_dir(abs) else {
            return;
        };
        let mut names: Vec<String> = read
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        for name in names {
            if rel.is_empty() && name == ".git" {
                continue;
            }
            let path = abs.join(&name);
            let Ok(meta) = fs::symlink_metadata(&path) else {
                continue;
            };
            if meta_is_link(&meta) {
                continue;
            }
            let child = join(rel, &name);
            if meta.is_dir() {
                self.folder(&path, &child, out);
            } else {
                self.file(&path, &child, out);
            }
        }
    }

    fn folder(&self, abs: &Path, rel: &str, out: &mut Dirt) {
        let prefix = format!("{rel}/");
        if self.tracked.keys().any(|k| k.starts_with(&prefix)) {
            self.dir(abs, rel, out);
            return;
        }
        if self.patterns.ignored(rel, true) {
            return;
        }
        if abs.join(".git").exists() {
            out.lines.push(format!("?? {prefix}"));
            out.others.push(prefix);
            return;
        }
        let mut inner = Dirt::default();
        self.dir(abs, rel, &mut inner);
        if !inner.others.is_empty() {
            out.lines.push(format!("?? {prefix}"));
            out.others.extend(inner.others);
        }
    }

    fn file(&self, abs: &Path, rel: &str, out: &mut Dirt) {
        match self.tracked.get(rel) {
            Some(text) => {
                if fs::read(abs).ok().as_deref() != Some(text.as_bytes()) {
                    out.lines.push(format!(" M {rel}"));
                    out.changed.push(rel.to_string());
                }
            }
            None => {
                if !self.patterns.ignored(rel, false) {
                    out.lines.push(format!("?? {rel}"));
                    out.others.push(rel.to_string());
                }
            }
        }
    }
}

/// The disk of a worktree folder against the files its HEAD tracks.
pub(super) fn dirt(
    worktree: &str,
    patterns: &Patterns,
    tracked: &BTreeMap<String, String>,
) -> Dirt {
    let mut out = Dirt::default();
    Walk { patterns, tracked }.dir(Path::new(worktree), "", &mut out);
    for path in tracked.keys() {
        if fs::symlink_metadata(Path::new(worktree).join(path)).is_err() {
            out.lines.push(format!(" D {path}"));
            out.changed.push(path.clone());
        }
    }
    out
}
