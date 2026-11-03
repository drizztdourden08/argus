//! Putting the launch flags to work on a bundle (`applyLaunch`): link the named launcher
//! (`--launcher`), unlink (`--cmd ""`), or write the launcher named after the tag and link it
//! (`--cmd`, `--cwd`, `--window`, or a legacy command moved over). The bundle record carries no
//! command of its own afterwards, except a legacy one that cannot move safely.

use argus_board::Board;
use argus_board::json::{doc, set, truthy, unset};
use argus_launchers::launcher::{
    LauncherPatch, WriteContext, launcher_def_name, launcher_slug, retire_launcher, write_launcher,
};
use serde_json::Value;

use super::parts::Parts;
use super::support::bundle_mut;
use crate::cli::exit::CoordError;
use crate::verbs::agent::base::links::linked;
use crate::verbs::agent::base::serves::{check_link, plan_of};
use crate::verbs::agent::rules;
use crate::who::Caller;

/// What the flags changed on the bundle and the notes to print after the line.
#[derive(Default)]
pub struct Applied {
    /// The fields changed, in order.
    pub changed: Vec<String>,
    /// Short sentences about what else happened.
    pub notes: Vec<String>,
}

/// The command a bundle written before launchers carries itself.
struct Legacy {
    cmd: String,
    cwd: Option<String>,
    window: bool,
}

struct Step<'a> {
    board: &'a mut Board,
    who: &'a Caller,
    tag: &'a str,
    cwd: &'a str,
    legacy: Option<Legacy>,
    done: Applied,
}

/// `bundleDefName`: the run name a bundle's own Play had, `bundle-<slug>` cut to 40 characters.
fn bundle_def_name(tag: &str) -> Option<String> {
    let lower = tag.to_lowercase();
    let mut slug = String::new();
    for c in lower.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            slug.push(c);
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        return None;
    }
    let name: String = format!("bundle-{slug}").chars().take(40).collect();
    Some(name.trim_end_matches('-').to_owned())
}

/// A bundle written before launchers: its own cmd, cwd and window go, with the definition
/// `bundle-<tag>` they made (only when that definition is the bundle's own). Returns a note, or an
/// empty text.
pub fn drop_legacy(board: &mut Board, tag: &str) -> String {
    if let Some(rec) = argus_board::json::doc_mut(board, "builds", tag) {
        for key in ["cmd", "cwd", "window"] {
            unset(rec, key);
        }
    }
    let Some(name) = bundle_def_name(tag) else {
        return String::new();
    };
    let own = doc(board, "runDefs", &name).is_some_and(|d| d["bundle"].as_str() == Some(tag));
    if !own {
        return String::new();
    }
    argus_board::json::map_mut(board, "runDefs").shift_remove(&name);
    format!("old Play {name} removed")
}

/// A launcher the bundle no longer starts is retired when it was made for this bundle
/// (`coord build --cmd`) and no other bundle links it. Returns a note, or an empty text.
///
/// # Errors
/// The launcher rules refuse the retirement.
pub fn retire_orphan(
    board: &mut Board,
    who: &Caller,
    tag: &str,
    name: Option<&str>,
) -> Result<String, CoordError> {
    let Some(name) = name else {
        return Ok(String::new());
    };
    let kept = doc(board, "launchers", name).is_none_or(|l| {
        truthy(l, "retiredAt")
            || l["bundle"].as_str() != Some(tag)
            || argus_board::served::is_master(board, name)
    });
    if kept || !linked(board, name, Some(tag)).is_empty() {
        return Ok(String::new());
    }
    let note = format!("bundle {tag} no longer starts it");
    rules::run(board, who, |b, w, at| {
        retire_launcher(b, w, name, Some(&note), at)
    })?;
    Ok(format!("launcher {name} retired"))
}

impl Step<'_> {
    fn mark(&mut self, key: &str) {
        if !self.done.changed.iter().any(|k| k == key) {
            self.done.changed.push(key.to_owned());
        }
    }

    fn launcher(&self) -> Option<String> {
        doc(self.board, "builds", self.tag).and_then(|b| b["launcher"].as_str().map(str::to_owned))
    }

    fn relink(&mut self, name: Option<&str>) -> Result<(), CoordError> {
        let was = self.launcher();
        if was.as_deref() == name {
            return Ok(());
        }
        set(bundle_mut(self.board, self.tag)?, "launcher", name);
        self.mark("launcher");
        let gone = retire_orphan(self.board, self.who, self.tag, was.as_deref())?;
        if !gone.is_empty() {
            self.done.notes.push(gone);
        }
        Ok(())
    }

    fn forget(&mut self) {
        if self.legacy.is_none() {
            return;
        }
        self.mark("cmd");
        let note = drop_legacy(self.board, self.tag);
        if !note.is_empty() {
            self.done.notes.push(note);
        }
    }

    /// The launcher the flags name, or none: the bundle follows it.
    fn link(&mut self, link: Option<&str>) -> Result<(), CoordError> {
        if let Some(name) = link {
            let found = doc(self.board, "launchers", name);
            let Some(found) = found else {
                return Err(CoordError::refuse(format!(
                    "no launcher \"{name}\": coord launcher set {name} --cmd \"<command>\" makes it."
                )));
            };
            if truthy(found, "retiredAt") {
                return Err(CoordError::refuse(format!(
                    "launcher {name} is retired: bring it back with coord launcher set {name}, or link another."
                )));
            }
        }
        self.relink(link)?;
        self.forget();
        Ok(())
    }

    /// The item the launcher written by this call serves: the one `--for` names, else for a new
    /// launcher the one test plan that names the bundle. A launcher that exists keeps its link.
    fn serve_target(&self, own: &str, parts: &Parts) -> Result<Option<String>, CoordError> {
        if let Some(given) = &parts.for_item {
            return given.clone().map(Some).ok_or_else(|| {
                CoordError::refuse(
                    "--for takes an item id such as I-12: a build always serves one.",
                )
            });
        }
        if doc(self.board, "launchers", own).is_some() {
            return Ok(None);
        }
        Ok(plan_of(self.board, self.tag))
    }

    /// `--for` alone: the item the bundle's launcher serves.
    fn change_for(&mut self, parts: &Parts) -> Result<(), CoordError> {
        let Some(name) = self.launcher() else {
            return Err(CoordError::refuse(format!(
                "bundle {} has no launcher yet: pass --cmd \"<command>\" with --for, or link one with --launcher <name>.",
                self.tag
            )));
        };
        let patch = LauncherPatch {
            for_item: parts.for_item.clone(),
            ..LauncherPatch::default()
        };
        check_link(self.board, &name, &patch)?;
        let cwd = self.cwd;
        let written = rules::run(self.board, self.who, |b, w, at| {
            write_launcher(
                b,
                w,
                &name,
                &patch,
                &WriteContext {
                    cwd,
                    bundle: None,
                    at,
                },
            )
        })?;
        if written.changed.contains(&"for") {
            self.mark("for");
        }
        Ok(())
    }

    /// Writes the launcher named after the tag (the flags given, over a legacy command when there
    /// is one) and links it.
    fn move_to(&mut self, own: &str, parts: &Parts) -> Result<(), CoordError> {
        let mut patch = parts.launch.patch();
        if let Some(legacy) = &self.legacy {
            if patch.cmd.is_none() {
                patch.cmd = Some(Some(legacy.cmd.clone()));
            }
            if parts.launch.cwd.is_none() && legacy.cwd.as_deref().is_some_and(|c| !c.is_empty()) {
                patch.cwd = Some(legacy.cwd.clone());
            }
            if parts.launch.window.is_none() && legacy.window {
                patch.window = Some(true);
            }
        }
        patch.for_item = self.serve_target(own, parts)?.map(Some);
        check_link(self.board, own, &patch)?;
        let (tag, cwd) = (self.tag, self.cwd);
        let written = rules::run(self.board, self.who, |b, w, at| {
            write_launcher(
                b,
                w,
                own,
                &patch,
                &WriteContext {
                    cwd,
                    bundle: Some(tag),
                    at,
                },
            )
        })?;
        let marked: Vec<String> = if written.created {
            parts.launch.keys().into_iter().map(str::to_owned).collect()
        } else {
            written.changed.iter().map(|k| (*k).to_owned()).collect()
        };
        for key in marked {
            self.mark(&key);
        }
        if written.changed.contains(&"for") {
            self.mark("for");
        }
        if let Some(Some(item)) = &patch.for_item {
            self.done.notes.push(format!("serves {item}"));
        }
        if written.created {
            let how = if self.legacy.is_some() && parts.launch.cmd.is_none() {
                "made from its own launch command"
            } else {
                "set"
            };
            self.done.notes.push(format!("launcher {own} {how}"));
        }
        if !written.play.is_empty() {
            self.done.notes.push(
                written
                    .play
                    .strip_suffix('.')
                    .unwrap_or(&written.play)
                    .to_owned(),
            );
        }
        self.relink(Some(own))?;
        self.forget();
        Ok(())
    }

    /// No flag, a legacy command: it moves over only when that is safe: a launcher of that name,
    /// a bundle linking it or a run definition under its Play name would change what something
    /// else starts.
    fn move_legacy(&mut self, own: &str, parts: &Parts) -> Result<(), CoordError> {
        let target = doc(self.board, "launchers", own).is_some();
        let taken = doc(self.board, "runDefs", &launcher_def_name(own))
            .is_some_and(|d| d["launcher"].as_str() != Some(own));
        let serves_nothing = plan_of(self.board, self.tag).is_none();
        if serves_nothing {
            self.done.notes.push(format!(
                "its launch command stays on the bundle (no single test plan names it): pass --for I-n with --cmd to move it to launcher {own}"
            ));
            return Ok(());
        }
        if target || !linked(self.board, own, Some(self.tag)).is_empty() || taken {
            self.done.notes.push(format!(
                "its launch command stays on the bundle (a launcher or run named {own} exists already): link one with --launcher <name>"
            ));
            return Ok(());
        }
        self.move_to(own, parts)
    }

    /// The refusal of a new launcher that would serve nothing.
    fn check_serves(&self, own: &str, parts: &Parts) -> Result<(), CoordError> {
        let new = doc(self.board, "launchers", own).is_none();
        if new && parts.for_item.is_none() && plan_of(self.board, self.tag).is_none() {
            return Err(CoordError::refuse(format!(
                "a build serves an item and bundle {} names none: pass --for I-n with --cmd (no single test plan names the bundle).",
                self.tag
            )));
        }
        Ok(())
    }

    /// The refusals before a write the flags asked for.
    fn check_write(&self, own: &str, parts: &Parts) -> Result<(), CoordError> {
        self.check_serves(own, parts)?;
        let others = linked(self.board, own, Some(self.tag));
        if !others.is_empty() {
            return Err(CoordError::refuse(format!(
                "launcher {own} is already linked to bundle {}: pick another tag, or link it with --launcher {own} and change it with coord launcher set {own}.",
                others.join(", ")
            )));
        }
        let target = doc(self.board, "launchers", own).is_some();
        let current = self.launcher();
        let has_cmd = parts.launch.cmd.is_some();
        if !has_cmd && self.legacy.is_none() && !(target && current.as_deref() == Some(own)) {
            return Err(CoordError::refuse(match current {
                Some(c) => format!(
                    "bundle {} starts launcher {c}: change its folder or window with coord launcher set {c} --cwd ... | --window.",
                    self.tag
                ),
                None => format!(
                    "bundle {} has no launcher yet: pass --cmd \"<command>\" too, or link one with --launcher <name>.",
                    self.tag
                ),
            }));
        }
        if self.legacy.is_some() && !has_cmd && target {
            return Err(CoordError::refuse(format!(
                "a launcher {own} exists already: link it with --launcher {own}, or give the command with --cmd."
            )));
        }
        Ok(())
    }

    fn apply(&mut self, parts: &Parts) -> Result<(), CoordError> {
        if let Some(link) = &parts.link {
            return self.link(link.as_deref());
        }
        if matches!(parts.launch.cmd, Some(None)) {
            self.relink(None)?;
            self.forget();
            return Ok(());
        }
        if parts.for_item.is_some() && parts.launch.keys().is_empty() {
            return self.change_for(parts);
        }
        let touched = !parts.launch.keys().is_empty();
        if !touched && self.legacy.is_none() {
            return Ok(());
        }
        let Some(own) = launcher_slug(self.tag) else {
            if !touched {
                return Ok(());
            }
            return Err(CoordError::refuse(format!(
                "the tag {} has no letter or digit, so its launcher has no name: use a tag such as bundle-4, or link a launcher with --launcher <name>.",
                self.tag
            )));
        };
        if !touched {
            return self.move_legacy(&own, parts);
        }
        self.check_write(&own, parts)?;
        self.move_to(&own, parts)
    }
}

fn legacy_of(board: &Board, tag: &str) -> Option<Legacy> {
    let rec = doc(board, "builds", tag)?;
    let cmd = rec["cmd"].as_str().filter(|c| !c.is_empty())?;
    Some(Legacy {
        cmd: cmd.to_owned(),
        cwd: rec["cwd"].as_str().map(str::to_owned),
        window: rec["window"] == Value::Bool(true),
    })
}

/// `applyLaunch`: what the flags do to the bundle `tag` and its launcher.
///
/// # Errors
/// A launcher that does not exist, is retired or is linked elsewhere, a tag with no letters or
/// digits, or a refusal of the launcher rules.
pub fn apply_launch(
    board: &mut Board,
    who: &Caller,
    (tag, cwd): (&str, &str),
    parts: &Parts,
) -> Result<Applied, CoordError> {
    let legacy = legacy_of(board, tag);
    let mut step = Step {
        board,
        who,
        tag,
        cwd,
        legacy,
        done: Applied::default(),
    };
    step.apply(parts)?;
    Ok(step.done)
}
