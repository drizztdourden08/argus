//! `coord proc kill <pid> [--tree] [--reason "..."]` from a project's thread: ends a process of
//! the thread's own project (its root, its worktrees or its ci root), and with `--tree` the
//! whole tree. Never a Claude session, the dashboard, another project or a system process: the
//! dashboard checks that on a fresh sample and writes the activity line.

use argus_core::js;
use serde_json::{Value, json};

use super::dashboard;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::verbs::remote::support::scope::require_board;
use crate::verbs::remote::support::text::{cell, collapse};
use crate::who::{caller, find_project};

const NO_PROJECT: &str = "this session belongs to no project: coord proc kill ends only a process of the thread's own project (its root, its worktrees or its ci root).";

fn line(out: &Value, pid: i64, tree: bool) -> String {
    let more = out["tree"].as_array().map_or(0, Vec::len);
    let and_tree = match (tree, more) {
        (false, _) => String::new(),
        (true, 0) => " and its tree".to_owned(),
        (true, n) => format!(" and its tree ({n} more)"),
    };
    format!("killed {} {pid}{and_tree}.", cell(out.get("name")))
}

/// The thread's kill of its own process `pid`.
pub(super) fn kill(call: &mut Call<'_>, pid: i64) -> Outcome {
    let env = &call.inv.env;
    let project = find_project(env, &call.inv.cwd, &call.options, env.session_id())
        .map_err(|_| CoordError::refuse(NO_PROJECT))?;
    let board = require_board(call, &project)?;
    let who = caller(&board, &call.inv.cwd, env, &call.options)?;
    let reason = call
        .options
        .text("reason")
        .map(collapse)
        .filter(|r| !r.is_empty());
    let tree = call.options.flag("tree");
    let body = json!({ "by": who.thread, "tree": tree, "reason": reason });
    let path = format!("/api/p/{project}/procs/{pid}/kill");
    let out = dashboard(call)?.post(&path, &body)?;
    if !js::truthy(&out["killed"]) {
        return Err(CoordError::refuse(format!(
            "could not kill {} {pid}: {}",
            cell(out.get("name")),
            cell(out.get("error"))
        )));
    }
    let said = line(&out, pid, tree);
    call.inv.say(&[&said]);
    Ok(())
}
