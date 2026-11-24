//! Play: a launcher that is not retired and whose command is one line also gets the run
//! definition `launcher-<name>` in `board.runDefs` (hand, kind launch, the built-in terminal
//! unless `window`), kept in step by every write. `launcher` on the definition names its
//! owner, so a definition made by hand under that name is never touched.

use argus_core::lead::LEAD_ROLE;
use std::path::{Component, Path, PathBuf};

use super::join::is_live;
use super::naming::launcher_def_name;
use super::record::{Launcher, LauncherBoard, RuleError, Who};
use crate::def::{RunDef, RunKind};

/// The folder the project lives in: the lead's thread root, else the caller's own, else
/// where the CLI runs.
pub fn project_root(board: &LauncherBoard, who: &Who, cwd: &str) -> String {
    board
        .threads
        .values()
        .find(|t| t.role == LEAD_ROLE && t.root.is_some() && t.deleted_at.is_none())
        .and_then(|t| t.root.clone())
        .or_else(|| who.thread.as_ref().and_then(|t| t.root.clone()))
        .unwrap_or_else(|| cwd.to_owned())
}

/// `path.resolve(root, rel)`: an absolute `rel` wins, `.` and `..` are folded.
pub fn resolve_path(root: &str, rel: &str) -> String {
    let joined = Path::new(root).join(rel);
    let mut out = PathBuf::new();
    for part in joined.components() {
        match part {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out.to_string_lossy().into_owned()
}

fn one_line(cmd: &str) -> bool {
    !cmd.contains(['\0', '\r', '\n'])
}

fn missing_folder_note(dir: &str) -> &'static str {
    if Path::new(dir).is_dir() {
        ""
    } else {
        "; that folder does not exist yet, so Play will fail until it does"
    }
}

fn taken_error(name: &str, have: &RunDef) -> RuleError {
    let owner = have.bundle.as_ref().map_or_else(
        || "a launch defined with coord run define".to_owned(),
        |b| format!("bundle {b}"),
    );
    RuleError::new(format!(
        "the run name {name} is already taken by {owner}: pick another launcher name."
    ))
}

fn play_def(who: &Who, rec: &Launcher, cmd: &str, dir: &str, rev: u32, at: &str) -> RunDef {
    RunDef {
        name: launcher_def_name(&rec.name),
        label: Some(format!("Play {}", rec.name)),
        launcher: Some(rec.name.clone()),
        cmd: cmd.to_owned(),
        cwd: dir.to_owned(),
        kind: RunKind::Launch,
        hand: true,
        visible: rec.window,
        window: rec.window,
        timeout_sec: 0,
        parallel: 1,
        defined_by: who.actor.clone(),
        defined_at: at.to_owned(),
        rev,
        ..RunDef::default()
    }
}

/// Keeps `board.runDefs` in step with the launcher `name`: written while it is live and its
/// command is one line, removed otherwise. Returns a short note for the verb's output, or
/// an empty string when nothing changed.
pub fn sync_launcher_play(
    board: &mut LauncherBoard,
    who: &Who,
    name: &str,
    cwd: Option<&str>,
    at: &str,
) -> Result<String, RuleError> {
    let Some(rec) = board.launchers.get(name).cloned() else {
        return Ok(String::new());
    };
    let def_name = launcher_def_name(name);
    let have = board.run_defs.get(&def_name).cloned();
    let live = is_live(Some(&rec));
    if let Some(other) = have
        .as_ref()
        .filter(|h| h.launcher.as_deref() != Some(name))
    {
        return if live {
            Err(taken_error(&def_name, other))
        } else {
            Ok(String::new())
        };
    }
    let cmd = rec.cmd.clone().unwrap_or_default();
    if !(live && !cmd.is_empty() && one_line(&cmd)) {
        return Ok(remove_play(
            board,
            &def_name,
            have.is_some(),
            live && !cmd.is_empty(),
        ));
    }
    let root = project_root(board, who, cwd.unwrap_or("."));
    let dir = resolve_path(&root, rec.cwd.as_deref().unwrap_or("."));
    if have
        .as_ref()
        .is_some_and(|h| h.cmd == cmd && h.cwd == dir && h.window == rec.window)
    {
        return Ok(String::new());
    }
    let rev = have.as_ref().map_or(1, |h| h.rev + 1);
    board
        .run_defs
        .insert(def_name.clone(), play_def(who, &rec, &cmd, &dir, rev, at));
    let window = if rec.window {
        ", with a console window"
    } else {
        ""
    };
    Ok(format!(
        "Play {} ({def_name}, in {dir}{window}{}).",
        if have.is_some() { "updated" } else { "ready" },
        missing_folder_note(&dir)
    ))
}

fn remove_play(board: &mut LauncherBoard, def_name: &str, had: bool, multi_line: bool) -> String {
    if !had {
        return if multi_line {
            "no Play button: the command spans several lines, a run definition is one line."
                .to_owned()
        } else {
            String::new()
        };
    }
    board.run_defs.remove(def_name);
    format!("Play removed ({def_name}).")
}
