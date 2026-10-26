//! `coord run define | undefine | defs`: the launches a lead lets others start, kept on the
//! board file alone. Only a name ever reaches the server; the command line, the folder and the
//! environment stay on the board.

use std::path::Path;

use argus_board::Board;
use argus_board::json::{map_mut, truthy};
use argus_core::clock::now_iso;
use argus_core::js;
use serde_json::{Map, Value, json};

use super::names::{int_option, need_name};
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::resolve::resolve;
use crate::cli::route::Call;
use crate::cli::{Val, out};
use crate::verbs::remote::support::collate;
use crate::verbs::remote::support::link::project_of;
use crate::verbs::remote::support::scope::{Scope, locked, require_board};
use crate::verbs::remote::support::text::{cell, say_all};
use crate::verbs::shared::caller::is_coordinator;
use crate::who::Caller;

const KINDS: [&str; 4] = ["build", "test", "launch", "task"];

/// A launch as typed: every argument checked, nothing touched yet.
struct Def {
    name: String,
    cmd: String,
    dir: String,
    kind: String,
    hand: bool,
    visible: bool,
    timeout: i64,
    parallel: i64,
    env: Map<String, Value>,
}

fn read_cmd(call: &Call<'_>) -> Result<String, CoordError> {
    let cmd = call.options.text("cmd").map_or("", js::trim).to_owned();
    if cmd.is_empty() {
        return Err(CoordError::refuse("--cmd \"<command line>\" is required."));
    }
    if js::len(&cmd) > 2000 || cmd.contains(['\0', '\r', '\n']) {
        return Err(CoordError::refuse(
            "--cmd must be one line of at most 2000 characters.",
        ));
    }
    Ok(cmd)
}

fn read_kind(call: &Call<'_>) -> Result<String, CoordError> {
    let kind = match call.options.get("kind") {
        None => Some("task"),
        Some(_) => call.options.text("kind"),
    };
    match kind.filter(|k| KINDS.contains(k)) {
        Some(kind) => Ok(kind.to_owned()),
        None => Err(CoordError::refuse(format!(
            "--kind must be one of {}.",
            KINDS.join(", ")
        ))),
    }
}

fn read_dir(call: &Call<'_>) -> Result<String, CoordError> {
    let typed = call.options.text("cwd").unwrap_or(".");
    let dir = resolve(&call.inv.cwd, Path::new(typed));
    if !dir.is_dir() {
        return Err(CoordError::refuse(format!(
            "--cwd {} is not a folder.",
            dir.display()
        )));
    }
    Ok(dir.to_string_lossy().into_owned())
}

fn read_env(call: &Call<'_>) -> Result<Map<String, Value>, CoordError> {
    let mut env = Map::new();
    for val in call.options.many("env") {
        let pair = match val {
            Val::Text(text) => text,
            Val::Flag => "true".to_owned(),
        };
        let at = pair.find('=').unwrap_or(0);
        let key = &pair[..at];
        let valid = key
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        if !valid {
            return Err(CoordError::refuse(format!(
                "--env takes KEY=VALUE, got \"{pair}\"."
            )));
        }
        env.insert(key.to_owned(), json!(&pair[at + 1..]));
    }
    Ok(env)
}

fn read_def(call: &Call<'_>) -> Result<Def, CoordError> {
    let name = need_name(call.positional.first(), "name")?;
    let cmd = read_cmd(call)?;
    let kind = read_kind(call)?;
    let hand = call.options.flag("hand");
    let visible = call.options.flag("visible");
    if visible && !hand {
        return Err(CoordError::refuse(
            "--visible only goes with --hand: launches are hidden, only the build handed to the owner may open a window.",
        ));
    }
    let dir = read_dir(call)?;
    let fallback = if kind == "build" || kind == "test" {
        1800
    } else {
        0
    };
    let timeout = int_option(&call.options, "timeout", fallback, (0, 86400))?;
    let parallel = int_option(&call.options, "parallel", 1, (1, 8))?;
    let env = read_env(call)?;
    Ok(Def {
        name,
        cmd,
        dir,
        kind,
        hand,
        visible,
        timeout,
        parallel,
        env,
    })
}

fn summary(def: &Def, before: bool) -> String {
    let keys: Vec<&str> = def.env.keys().map(String::as_str).collect();
    let hand = if def.hand { ", hand" } else { "" };
    let visible = if def.hand && def.visible {
        " (visible)"
    } else {
        ""
    };
    let timeout = if def.timeout == 0 {
        "none".to_owned()
    } else {
        format!("{}s", def.timeout)
    };
    let env = if keys.is_empty() {
        String::new()
    } else {
        format!(", env {}", keys.join(", "))
    };
    format!(
        "{} {}: {}{hand}{visible}, cwd {}, timeout {timeout}, parallel {}{env}",
        if before { "updated" } else { "defined" },
        def.name,
        def.kind,
        def.dir,
        def.parallel,
    )
}

fn record(def: &Def, actor: &str, rev: i64) -> Value {
    json!({
        "name": def.name, "cmd": def.cmd, "cwd": def.dir, "kind": def.kind, "env": def.env,
        "hand": def.hand, "visible": def.hand && def.visible, "timeoutSec": def.timeout,
        "parallel": def.parallel, "definedBy": actor, "definedAt": now_iso(), "rev": rev,
    })
}

/// The caller of a change that is the coordinator's alone.
fn coordinator(board: &mut Board, scope: &Scope<'_>, refusal: &str) -> Result<Caller, CoordError> {
    let who = scope.begin(board)?;
    if is_coordinator(&who) {
        Ok(who)
    } else {
        Err(CoordError::refuse(refusal))
    }
}

/// `coord run define <name> --cmd "<command line>" ...`.
pub(super) fn define(call: &mut Call<'_>) -> Outcome {
    let def = read_def(call)?;
    let project = project_of(call).to_owned();
    locked(call, &project, false, |board, scope| {
        let who = coordinator(
            board,
            scope,
            "only the coordinator can define what may be launched. Ask with coord inbox send coordinator \"...\" or coord message.",
        )?;
        let defs = map_mut(board, "runDefs");
        let before = defs.get(&def.name).cloned();
        let rev = before.as_ref().and_then(|b| b["rev"].as_i64()).unwrap_or(0) + 1;
        defs.insert(def.name.clone(), record(&def, &who.actor, rev));
        scope.say(&[&summary(&def, before.is_some())]);
        Ok(())
    })
}

/// `coord run undefine <name>`.
pub(super) fn undefine(call: &mut Call<'_>) -> Outcome {
    let name = need_name(call.positional.first(), "name")?;
    let project = project_of(call).to_owned();
    locked(call, &project, false, |board, scope| {
        coordinator(
            board,
            scope,
            "only the coordinator can remove a launch definition.",
        )?;
        if map_mut(board, "runDefs").shift_remove(&name).is_none() {
            return Err(CoordError::refuse(format!(
                "no launch \"{name}\" is defined."
            )));
        }
        scope.say(&[&format!("removed {name}")]);
        Ok(())
    })
}

/// Env values are secrets; the keys only.
fn shown(def: &Value) -> Value {
    let mut shown = def.clone();
    let hidden: Map<String, Value> = def["env"]
        .as_object()
        .map(|env| env.keys().map(|k| (k.clone(), json!("(hidden)"))).collect())
        .unwrap_or_default();
    shown["env"] = Value::Object(hidden);
    shown
}

fn row(def: &Value) -> Vec<String> {
    let flags: Vec<&str> = [("hand", "hand"), ("visible", "visible")]
        .iter()
        .filter(|(field, _)| truthy(def, field))
        .map(|(_, word)| *word)
        .collect();
    let timeout = if js::truthy(&def["timeoutSec"]) {
        format!("{}s", cell(def.get("timeoutSec")))
    } else {
        "none".to_owned()
    };
    vec![
        cell(def.get("name")),
        cell(def.get("kind")),
        flags.join(","),
        timeout,
        cell(def.get("parallel")),
        out::clip(&cell(def.get("cmd")), 70),
    ]
}

/// `coord run defs [--json]`.
pub(super) fn defs(call: &mut Call<'_>) -> Outcome {
    let board = require_board(call, project_of(call))?;
    let mut list: Vec<Value> = board["runDefs"]
        .as_object()
        .map(|m| m.values().cloned().collect())
        .unwrap_or_default();
    list.sort_by(|a, b| collate::compare(&cell(a.get("name")), &cell(b.get("name"))));
    if call.options.flag("json") {
        let shown: Vec<Value> = list.iter().map(shown).collect();
        out::as_json(&mut call.inv.stdout, &json!({ "defs": shown }));
        return Ok(());
    }
    if list.is_empty() {
        call.inv.say(&["no launches defined. A coordinator adds one with: coord run define <name> --cmd \"<command line>\""]);
        return Ok(());
    }
    let mut rows = vec![
        ["name", "kind", "flags", "timeout", "parallel", "command"]
            .map(str::to_owned)
            .to_vec(),
    ];
    rows.extend(list.iter().map(row));
    say_all(call.inv, &[out::table(&rows)]);
    Ok(())
}
