//! `board shortcut`: one configuration in a checkout's `.claude/launch.json` that the Browser
//! pane of Claude Desktop lists on a new tab, so Play opens the board and nothing is started. The
//! same name replaces itself, so it is idempotent. The Overlord gets two rows, "Argus" (the
//! overview) and "Overlord" (its own board).

use std::path::PathBuf;

use argus_board::json::{display, truthy};
use serde_json::Value;

use crate::cli::exit::{CoordError, Outcome};
use crate::cli::resolve::resolve;
use crate::client::board_port;
use crate::shortcut::{shortcut_configs, write_shortcuts};
use crate::verbs::thread::docs::id_of;
use crate::verbs::thread::frame::Frame;
use crate::who::git_root;

/// The folder whose `.claude/launch.json` gets the shortcut: `--root`, else the checkout the
/// thread registered from (its root), else (a call for the caller itself only) the git checkout
/// the cwd is in. The first that is an existing folder; never a stray `.git` far above the cwd.
fn shortcut_root(frame: &Frame<'_>, thread: &Value, own: bool) -> Option<PathBuf> {
    let here = git_root(frame.cwd);
    let candidates = [
        frame
            .options
            .text("root")
            .map(|r| resolve(frame.cwd, std::path::Path::new(r))),
        thread["root"]
            .as_str()
            .filter(|r| !r.is_empty())
            .map(|r| resolve(frame.cwd, std::path::Path::new(r))),
        (own && here.join(".git").exists()).then_some(here),
    ];
    candidates.into_iter().flatten().find(|dir| dir.is_dir())
}

fn names_of(configs: &[Value]) -> String {
    let names: Vec<String> = configs
        .iter()
        .map(|c| format!("'{}'", display(&c["name"])))
        .collect();
    names.join(" and ")
}

/// The verb `board shortcut [--thread <id>] [--root <dir>]`.
///
/// # Errors
/// Refused for an unknown or deleted thread, a thread with no folder, a thread id or project key
/// a host name cannot carry, a `launch.json` that cannot be merged, and no board.
pub fn run(frame: &mut Frame<'_>) -> Outcome {
    let board = frame.read_board()?;
    let asked = frame.options.text("thread");
    let named = asked.filter(|a| !a.is_empty());
    let thread = match named {
        Some(name) => board["threads"][name].clone(),
        None => board["threads"][frame.caller(&board)?.thread.as_str()].clone(),
    };
    if thread.is_null() || truthy(&thread, "deletedAt") {
        return Err(CoordError::refuse(format!(
            "no thread named \"{}\" on the {} board.",
            asked.unwrap_or("null"),
            frame.project
        )));
    }
    let root = shortcut_root(frame, &thread, named.is_none()).ok_or_else(|| {
        CoordError::refuse(format!(
            "no project folder to write .claude/launch.json into (thread {} has no root here). Pass --root <dir>.",
            id_of(&thread)
        ))
    })?;
    let label = board["name"]
        .as_str()
        .map_or_else(|| display(&board["project"]), str::to_owned);
    let port = board_port(frame.env, frame.paths);
    let configs = shortcut_configs(&frame.paths.coord_home, &label, frame.project, &id_of(&thread), port)?;
    let written = write_shortcuts(&root, &configs)?;
    let file = written.file.display();
    if !written.changed {
        let urls: Vec<String> = configs.iter().map(|c| display(&c["url"])).collect();
        frame.say(&[&format!(
            "{file} already lists {} ({}).",
            names_of(&configs),
            urls.join(", ")
        )]);
        return Ok(());
    }
    let mut lines = vec![format!("wrote {file}:")];
    for entry in &written.wrote {
        lines.push(format!("  {entry}"));
    }
    let play = if written.wrote.len() > 1 {
        "them"
    } else {
        "the board"
    };
    lines.push(format!(
        "In the Browser pane, a new tab now lists {}: Play opens {play} without starting anything.",
        names_of(&written.wrote)
    ));
    let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    frame.say(&refs);
    Ok(())
}
