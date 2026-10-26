//! The launcher side of a board as the typed rules of `argus-launchers` read it: the records of
//! `launchers` and `builds`, the Play definitions of `runDefs`, the threads and the handed build.
//! The board itself stays JSON, so nothing the rules do not know is lost.

use argus_board::Board;
use argus_board::json::{section, truthy};
use argus_board::ops::Role;
use argus_launchers::RunDef;
use argus_launchers::launcher::{Bundle, Launcher, LauncherBoard, ThreadInfo, Who};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::cli::exit::CoordError;
use crate::who::Caller;

fn text(v: &Value, key: &str) -> Option<String> {
    v[key].as_str().map(str::to_owned)
}

/// A record the typed side can read: a flag stored as anything becomes its truthiness, and a text
/// the record leaves `null` is dropped so its default applies.
fn tidy(rec: &Value, flags: &[&str], texts: &[&str]) -> Value {
    let mut map: Map<String, Value> = rec.as_object().cloned().unwrap_or_default();
    for key in flags {
        if map.contains_key(*key) {
            map.insert((*key).to_owned(), Value::Bool(truthy(rec, key)));
        }
    }
    for key in texts {
        if map.get(*key).is_some_and(Value::is_null) {
            map.shift_remove(*key);
        }
    }
    Value::Object(map)
}

fn record<T: DeserializeOwned>(
    what: &str,
    name: &str,
    rec: &Value,
    flags: &[&str],
    texts: &[&str],
) -> Result<T, CoordError> {
    serde_json::from_value(tidy(rec, flags, texts))
        .map_err(|_| CoordError::refuse(format!("the {what} \"{name}\" record cannot be read.")))
}

fn thread_info(t: &Value) -> ThreadInfo {
    ThreadInfo {
        id: text(t, "id").unwrap_or_default(),
        role: text(t, "role").unwrap_or_default(),
        status: text(t, "status").unwrap_or_default(),
        root: text(t, "root"),
        placeholder: truthy(t, "placeholder"),
        deleted_at: text(t, "deletedAt").filter(|d| !d.is_empty()),
    }
}

/// A Play definition as far as the rules look at it (a launcher's name, a bundle, the command and
/// where it runs, its revision).
fn run_def(name: &str, def: &Value) -> RunDef {
    RunDef {
        name: name.to_owned(),
        cmd: text(def, "cmd").unwrap_or_default(),
        cwd: text(def, "cwd").unwrap_or_default(),
        launcher: text(def, "launcher").filter(|l| !l.is_empty()),
        bundle: text(def, "bundle").filter(|b| !b.is_empty()),
        window: truthy(def, "window"),
        rev: def["rev"]
            .as_u64()
            .and_then(|r| u32::try_from(r).ok())
            .unwrap_or(0),
        ..RunDef::default()
    }
}

/// The typed view of the board.
///
/// # Errors
/// A launcher or bundle record is not of a readable shape.
pub fn typed_board(board: &Board) -> Result<LauncherBoard, CoordError> {
    let mut typed = LauncherBoard {
        handed_build: text(board, "handedBuild"),
        handed_at: text(board, "handedAt"),
        ..LauncherBoard::default()
    };
    for (name, rec) in section(board, "launchers") {
        let launcher: Launcher = record(
            "launcher",
            name,
            rec,
            &["current", "window"],
            &["name", "kind", "createdAt", "updatedAt"],
        )?;
        typed.launchers.insert(name.clone(), launcher);
    }
    for (tag, rec) in section(board, "builds") {
        let bundle: Bundle = record("bundle", tag, rec, &["window"], &["tag"])?;
        typed.builds.insert(tag.clone(), bundle);
    }
    for (id, t) in section(board, "threads") {
        typed.threads.insert(id.clone(), thread_info(t));
    }
    for (name, def) in section(board, "runDefs") {
        if def.is_object() {
            typed.run_defs.insert(name.clone(), run_def(name, def));
        }
    }
    Ok(typed)
}

/// The caller as the launcher rules know it.
#[must_use]
pub fn rule_who(typed: &LauncherBoard, caller: &Caller) -> Who {
    Who {
        actor: caller.actor.clone(),
        role: match caller.role {
            Role::Coordinator => argus_launchers::launcher::Role::Coordinator,
            Role::Human => argus_launchers::launcher::Role::Human,
            Role::Thread => argus_launchers::launcher::Role::Thread,
        },
        thread: typed.threads.get(&caller.thread).cloned(),
    }
}
