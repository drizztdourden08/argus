//! The backup written before any work is discarded: a bundle of commits, a patch, the untracked files.
//!
//! Files go to the backup folder as `<project>-<branch or folder>-<shortsha>-<yyyymmddhhmm>` (UTC) with `.bundle`,
//! `.patch`, `.untracked.txt` and `.untracked/`. No backup, no removal.

use crate::clock::{DAY_MS, stamp_from_ms};
use crate::error::{ErrorCode, Result, WtError};
use crate::exec::GitOutput;
use crate::fact::Fact;
use crate::paths::{base_name, clean_path, join_path, parent_of};
use crate::service::Service;
use argus_core::fs::{copy_atomic, write_atomic};
use serde::Serialize;
use std::fs;
use std::path::Path;

/// Untracked files are copied when there are at most this many…
pub const COPY_MAX_FILES: usize = 2000;
/// …and at most this many bytes.
pub const COPY_MAX_BYTES: u64 = 50 * 1024 * 1024;

/// What was written, and how to bring it back.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Backup {
    pub dir: String,
    pub base: String,
    pub bundle: Option<String>,
    pub bundle_kind: Option<String>,
    pub patch: Option<String>,
    pub untracked: Option<String>,
    pub untracked_copy: Option<String>,
    pub untracked_count: usize,
    pub nested_copy: Option<String>,
    pub restore: Vec<String>,
}

/// What to back up.
#[derive(Clone, Copy, Debug)]
pub struct Need {
    pub bundle: bool,
    pub patch: bool,
}

fn safe_name(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        let ok = c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-');
        if ok {
            out.push(c);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let trimmed: String = out.trim_matches('-').chars().take(60).collect();
    if trimmed.is_empty() {
        "worktree".to_string()
    } else {
        trimmed
    }
}

fn refuse(what: &str, run: Option<&GitOutput>) -> WtError {
    let why = run
        .map(|r| format!(": {}", r.failure_text()))
        .unwrap_or_default();
    WtError::new(
        ErrorCode::BackupFailed,
        format!("the backup could not be written ({what}{why}), so nothing was removed"),
    )
}

fn backup_group(name: &str) -> String {
    for suffix in [".untracked.txt", ".untracked", ".bundle", ".patch"] {
        if let Some(base) = name.strip_suffix(suffix) {
            return base.to_string();
        }
    }
    name.to_string()
}

/// `<project>-<branch or folder>-<head7>`: the name of a backup of `fact` before its time stamp.
fn backup_base(project: &str, fact: &Fact) -> String {
    let head: String = fact
        .head
        .as_deref()
        .unwrap_or("nohead")
        .chars()
        .take(7)
        .collect();
    let name = fact.branch.clone().unwrap_or_else(|| base_name(&fact.path));
    format!("{}-{}-{head}", safe_name(project), safe_name(&name))
}

fn modified_ms(path: &Path) -> Option<i64> {
    fs::metadata(path)
        .ok()?
        .modified()
        .ok()
        .map(crate::clock::system_ms)
}

impl Service {
    fn backup_root(&self) -> String {
        clean_path(&self.opts.backup_dir)
    }

    /// Where a backup of `fact` written now goes, up to its time stamp: `<root>/<project>-<branch>-<head7>-`.
    pub(crate) fn backup_prefix(&self, project: &str, fact: &Fact) -> String {
        format!("{}/{}-", self.backup_root(), backup_base(project, fact))
    }

    /// Keep the newest `keep_backups`, and anything younger than `keep_backup_days`.
    pub fn prune_backups(&self) {
        let root = self.backup_root();
        let Ok(names) = fs::read_dir(&root) else {
            return;
        };
        let mut groups: std::collections::HashMap<String, (i64, Vec<String>)> =
            std::collections::HashMap::new();
        for entry in names.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(ms) = modified_ms(&entry.path()) else {
                continue;
            };
            let group = groups.entry(backup_group(&name)).or_insert((0, Vec::new()));
            group.0 = group.0.max(ms);
            group.1.push(name);
        }
        let mut sorted: Vec<(i64, Vec<String>)> = groups.into_values().collect();
        sorted.sort_by_key(|g| std::cmp::Reverse(g.0));
        let cutoff = self.now() - self.opts.keep_backup_days * DAY_MS;
        for (ms, files) in sorted.into_iter().skip(self.opts.keep_backups) {
            if ms >= cutoff {
                continue;
            }
            for file in files {
                let path = join_path(&root, &file);
                let _ = fs::remove_dir_all(&path).or_else(|_| fs::remove_file(&path));
            }
        }
    }

    fn write_bundle(&self, fact: &Fact, base: &str, out: &mut Backup) -> Result<()> {
        let file = join_path(&out.dir, &format!("{base}.bundle"));
        let reference = match (&fact.branch, fact.detached) {
            (Some(branch), false) => branch.clone(),
            _ => "HEAD".to_string(),
        };
        let git = &self.deps.git;
        let mut run = git.write(
            &fact.path,
            &["bundle", "create", &file, &reference, "--not", "--remotes"],
        );
        out.bundle_kind = Some("beyond the remotes".to_string());
        if !run.ok() {
            let _ = fs::remove_file(&file);
            run = git.write(&fact.path, &["bundle", "create", &file, &reference]);
            out.bundle_kind = Some("the whole branch".to_string());
        }
        if !run.ok() {
            return Err(refuse("git bundle create", Some(&run)));
        }
        let verify = git.write(
            fact.repo.as_deref().unwrap_or(&fact.path),
            &["bundle", "verify", &file],
        );
        if !verify.ok() {
            return Err(refuse("git bundle verify", Some(&verify)));
        }
        out.bundle = Some(file.clone());
        let target = if reference == "HEAD" {
            format!("restored-{}", safe_name(&base_name(&fact.path)))
        } else {
            reference.clone()
        };
        out.restore.push(format!(
            "git -C \"{}\" fetch \"{file}\" {reference}:{target}",
            fact.repo.as_deref().unwrap_or("<repo>")
        ));
        if out.bundle_kind.as_deref() == Some("the whole branch") {
            out.restore.push(format!("or git clone \"{file}\""));
        }
        Ok(())
    }

    /// The untracked files (not the ignored ones) of a worktree, and whether the backup can copy them.
    fn untracked_of(&self, fact: &Fact) -> Result<Untracked> {
        let listing = self.deps.git.write(
            &fact.path,
            &["ls-files", "--others", "--exclude-standard", "-z"],
        );
        if !listing.ok() {
            return Err(refuse("git ls-files", Some(&listing)));
        }
        let names: Vec<String> = listing
            .stdout
            .split('\0')
            .filter(|n| !n.is_empty())
            .map(String::from)
            .collect();
        let (files, bytes) = names.iter().fold((0, 0), |(f, b), n| {
            if f > COPY_MAX_FILES || b > COPY_MAX_BYTES {
                return (f, b);
            }
            let (more_files, more_bytes) = tree_size(&join_path(&fact.path, n));
            (f + more_files, b + more_bytes)
        });
        Ok(Untracked {
            names,
            files,
            bytes,
        })
    }

    /// When the untracked files are too many or too big to copy, the sentence that says they are only listed.
    pub(crate) fn untracked_warning(&self, fact: &Fact) -> Option<String> {
        let untracked = self.untracked_of(fact).ok()?;
        (!untracked.copy()).then(|| {
            format!(
                "its untracked files ({}) are only listed in the backup, not copied",
                untracked.size_text()
            )
        })
    }

    /// Why the backup of this worktree would only be partial, or none: its untracked files (`untracked`, the sentence
    /// of `untracked_warning` when it applies) or its nested repository are over the copy limits and would only be
    /// listed.
    pub(crate) fn partial_backup_why(fact: &Fact, untracked: Option<&str>) -> Option<String> {
        if let Some(why) = untracked {
            return Some(why.to_string());
        }
        let git_dir = fact.nested_git.as_deref()?;
        let folder = parent_of(git_dir);
        let (files, bytes) = tree_size(&folder);
        (fact.nested_too_big || files > COPY_MAX_FILES || bytes > COPY_MAX_BYTES).then(|| {
            format!(
                "its nested repository {folder} is over the copy limits and would only be listed"
            )
        })
    }

    fn write_untracked(&self, fact: &Fact, base: &str, out: &mut Backup) -> Result<()> {
        let untracked = self.untracked_of(fact)?;
        out.untracked_count = untracked.names.len();
        if untracked.names.is_empty() {
            return Ok(());
        }
        let copy = untracked.copy();
        let note = if copy {
            String::new()
        } else {
            format!(
                "# not copied: {} (over {COPY_MAX_FILES} files or {} MB)\n",
                untracked.size_text(),
                COPY_MAX_BYTES / 1_048_576
            )
        };
        let list = join_path(&out.dir, &format!("{base}.untracked.txt"));
        let write = || -> std::io::Result<()> {
            write_atomic(
                Path::new(&list),
                format!("{}\n{note}", untracked.names.join("\n")),
            )?;
            if copy {
                copy_files(
                    &fact.path,
                    &join_path(&out.dir, &format!("{base}.untracked")),
                    &untracked.names,
                )?;
            }
            Ok(())
        };
        write().map_err(|err| {
            refuse(
                &format!("the untracked files: {}", crate::error::io_code(&err)),
                None,
            )
        })?;
        out.untracked = Some(list);
        out.untracked_copy = copy.then(|| join_path(&out.dir, &format!("{base}.untracked")));
        Ok(())
    }

    fn write_patch(&self, fact: &Fact, base: &str, out: &mut Backup) -> Result<()> {
        let file = join_path(&out.dir, &format!("{base}.patch"));
        let diff = self.deps.git.write(
            &fact.path,
            &["diff", "HEAD", "--binary", &format!("--output={file}")],
        );
        if !diff.ok() || !Path::new(&file).exists() {
            return Err(refuse("git diff", Some(&diff)));
        }
        out.patch = Some(file.clone());
        self.write_untracked(fact, base, out)?;
        let head: String = fact
            .head
            .as_deref()
            .unwrap_or("its commit")
            .chars()
            .take(7)
            .collect();
        let back = out
            .untracked_copy
            .as_ref()
            .map(|c| format!(", then copy back {c}"))
            .unwrap_or_default();
        out.restore.push(format!(
            "git -C <a checkout of {head}> apply \"{file}\"{back}"
        ));
        Self::write_nested(fact, base, out)
    }

    /// A nested repository inside the worktree (it may hold real work, and a patch cannot): its folder is copied when it
    /// fits the copy limits, and named in the backup either way.
    fn write_nested(fact: &Fact, base: &str, out: &mut Backup) -> Result<()> {
        let Some(git_dir) = fact.nested_git.as_deref() else {
            return Ok(());
        };
        let folder = parent_of(git_dir);
        let (files, bytes) = tree_size(&folder);
        let list = join_path(&out.dir, &format!("{base}.nested.txt"));
        let copy = files <= COPY_MAX_FILES && bytes <= COPY_MAX_BYTES;
        let target = join_path(&out.dir, &format!("{base}.nested"));
        let note = if copy {
            "copied"
        } else {
            "not copied: over the copy limits"
        };
        let write = || -> std::io::Result<()> {
            write_atomic(Path::new(&list), format!("{folder}\n# {note}\n"))?;
            if copy {
                copy_path(&folder, &target)
            } else {
                Ok(())
            }
        };
        write().map_err(|err| {
            refuse(
                &format!("the nested repository: {}", crate::error::io_code(&err)),
                None,
            )
        })?;
        out.nested_copy = copy.then(|| target.clone());
        out.restore.push(format!(
            "the nested repository {folder} is in {}",
            if copy { target } else { list }
        ));
        Ok(())
    }

    /// Write the backup of what removing `fact` would lose; `backup_failed` when it cannot be written.
    pub(crate) fn write_backup(&self, project: &str, fact: &Fact, need: Need) -> Result<Backup> {
        let base = format!(
            "{}-{}",
            backup_base(project, fact),
            stamp_from_ms(self.now())
        );
        let dir = self.backup_root();
        let mut out = Backup {
            dir: dir.clone(),
            base: base.clone(),
            ..Backup::default()
        };
        fs::create_dir_all(&dir).map_err(|err| {
            refuse(
                &format!("the folder {dir}: {}", crate::error::io_code(&err)),
                None,
            )
        })?;
        if need.bundle {
            self.write_bundle(fact, &base, &mut out)?;
        }
        if need.patch {
            self.write_patch(fact, &base, &mut out)?;
        }
        self.prune_backups();
        Ok(out)
    }
}

/// The untracked files of a worktree: their names and how much there is.
struct Untracked {
    names: Vec<String>,
    files: usize,
    bytes: u64,
}

impl Untracked {
    fn copy(&self) -> bool {
        self.files <= COPY_MAX_FILES && self.bytes <= COPY_MAX_BYTES
    }

    fn size_text(&self) -> String {
        format!("{} files, {} MB", self.files, self.bytes / 1_048_576)
    }
}

/// Files and bytes under a path, a link counted as nothing and never entered; stops counting once past the copy limits.
fn tree_size(path: &str) -> (usize, u64) {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return (0, 0);
    };
    if crate::fs::meta_is_link(&meta) {
        return (0, 0);
    }
    if !meta.is_dir() {
        return (1, meta.len());
    }
    let (mut files, mut bytes) = (0, 0);
    for entry in fs::read_dir(path).into_iter().flatten().flatten() {
        let (f, b) = tree_size(&join_path(path, &entry.file_name().to_string_lossy()));
        (files, bytes) = (files + f, bytes + b);
        if files > COPY_MAX_FILES || bytes > COPY_MAX_BYTES {
            break;
        }
    }
    (files, bytes)
}

fn copy_files(from: &str, to: &str, names: &[String]) -> std::io::Result<()> {
    for name in names {
        copy_path(&join_path(from, name), &join_path(to, name))?;
    }
    Ok(())
}

/// Copy a file, or a folder with what is in it. A link is skipped, never followed.
fn copy_path(source: &str, target: &str) -> std::io::Result<()> {
    let meta = fs::symlink_metadata(source)?;
    if crate::fs::meta_is_link(&meta) {
        return Ok(());
    }
    if meta.is_dir() {
        fs::create_dir_all(target)?;
        for entry in fs::read_dir(source)?.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            copy_path(&join_path(source, &name), &join_path(target, &name))?;
        }
        return Ok(());
    }
    if let Some(parent) = Path::new(target).parent() {
        fs::create_dir_all(parent)?;
    }
    copy_atomic(Path::new(source), Path::new(target))
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_names_are_safe_and_groups_share_a_base() {
        assert_eq!(safe_name("claude/feat login"), "claude-feat-login");
        assert_eq!(safe_name("///"), "worktree");
        assert_eq!(safe_name(&"a".repeat(80)).len(), 60);
        assert_eq!(
            backup_group("alpha-b-abc-202610250000.untracked.txt"),
            "alpha-b-abc-202610250000"
        );
        assert_eq!(
            backup_group("alpha-b-abc-202610250000.bundle"),
            "alpha-b-abc-202610250000"
        );
    }
}
