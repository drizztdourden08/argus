//! `coord remote [status|local|remote]`, `coord remote ci-root <project> <dir> | --clear` and
//! `coord remote synced [--note "<what went out>"]`: the GitHub switch of
//! `<COORD_HOME>/remote.json`, the integration checkout of a project and the lead's daily full
//! remote sync. `local` freezes every call to GitHub, `remote` is the owner's go; only the owner
//! sets it. `synced` records `syncs.<project>`: Argus shows it on the board and wakes the lead
//! when a day passes without one.

use std::path::{Path, PathBuf};

use argus_core::remote::switch_file;
use argus_core::remote_state::{
    LOCAL, REMOTE, State, read_state, record_sync, set_ci_root, write_mode,
};

use crate::cli::exit::{CoordError, Outcome};
use crate::cli::resolve::resolve;
use crate::cli::route::Call;
use crate::who::find_project;

const USAGE: &str = "usage: coord remote [status|local|remote|synced|ci-root <project> <dir>].";
const CI_ROOT_USAGE: &str = "usage: coord remote ci-root <project> <dir> | --clear.";

fn describe(state: &State) -> String {
    let since = match state.since.as_deref().filter(|s| !s.is_empty()) {
        Some(at) => match state.by.as_deref().filter(|b| !b.is_empty()) {
            Some(by) => format!(" since {at} (set by {by})"),
            None => format!(" since {at}"),
        },
        None => " (no file: the default while the freeze lasts)".to_owned(),
    };
    let meaning = if state.mode == LOCAL {
        "GitHub is not used: no gh, no API, no push, no publish, no GitHub registry."
    } else {
        "GitHub is used again."
    };
    let roots = state
        .ci_roots
        .iter()
        .fold(String::new(), |all, (key, dir)| {
            format!(
                "{all}
  ci-root {key}: {}",
                js_text(dir)
            )
        });
    format!("remote mode: {}{since}. {meaning}{roots}", state.mode)
}

/// A JSON value as a JavaScript template literal prints it.
fn js_text(value: &serde_json::Value) -> String {
    value
        .as_str()
        .map_or_else(|| argus_core::js::string(value), str::to_owned)
}

fn home_of(call: &Call<'_>) -> PathBuf {
    call.inv.env.paths(&call.inv.cwd).coord_home
}

/// `--as` when it is a non-empty text, else the user name of the environment.
fn setter_of(call: &Call<'_>) -> Option<String> {
    call.options
        .text("as")
        .filter(|a| !a.is_empty())
        .or_else(|| call.inv.env.get("USERNAME"))
        .map(str::to_owned)
}

fn failed(e: &std::io::Error) -> CoordError {
    CoordError::refuse(e.to_string())
}

fn set(call: &mut Call<'_>, mode: &str) -> Outcome {
    let home = home_of(call);
    let by = setter_of(call);
    let state = write_mode(&home, mode, by.as_deref()).map_err(|e| failed(&e))?;
    let file = switch_file(&home);
    call.inv
        .say(&[&format!("{} ({})", describe(&state), file.display())]);
    Ok(())
}

/// The last full remote sync of each project, one line each.
fn sync_list(state: &State) -> String {
    if state.syncs.is_empty() {
        return "no remote sync recorded yet.".to_owned();
    }
    let rows = state.syncs.iter().fold(String::new(), |all, (key, sync)| {
        let at = sync
            .get("at")
            .map_or_else(|| "undefined".to_owned(), js_text);
        let note = sync
            .get("note")
            .and_then(serde_json::Value::as_str)
            .filter(|n| !n.is_empty())
            .map_or_else(String::new, |n| format!(" ({n})"));
        format!("{all}\n  {key}: {at}{note}")
    });
    format!("last remote sync per lead:{rows}")
}

fn status(call: &mut Call<'_>) {
    let state = read_state(&home_of(call));
    let text = format!("{}\n{}", describe(&state), sync_list(&state));
    call.inv.say(&[&text]);
}

/// `coord remote synced [--note "<what went out>"]`: the lead's daily full remote sync is done
/// (push, pull requests, packages, releases, deploys).
fn synced(call: &mut Call<'_>) -> Outcome {
    let env = &call.inv.env;
    let project = find_project(env, &call.inv.cwd, &call.options, env.session_id())?;
    let by = call
        .options
        .text("as")
        .filter(|a| !a.is_empty())
        .unwrap_or(&project);
    let note = call.options.text("note").unwrap_or_default();
    let home = home_of(call);
    let record = record_sync(&home, &project, Some(by), note).map_err(|e| failed(&e))?;
    let at = record["at"].as_str().unwrap_or_default();
    let tail = if note.is_empty() {
        String::new()
    } else {
        format!(": {}", record["note"].as_str().unwrap_or_default())
    };
    let line = format!("remote sync of {project} recorded at {at}{tail}. Due again in 24 h.");
    call.inv.say(&[&line]);
    Ok(())
}

/// A project key as the boards name them.
fn is_project(text: &str) -> bool {
    let mut chars = text.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn slashed(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn ci_root(call: &mut Call<'_>) -> Outcome {
    let project = call
        .positional
        .get(1)
        .filter(|p| is_project(p))
        .cloned()
        .ok_or_else(|| CoordError::refuse(CI_ROOT_USAGE))?;
    let home = home_of(call);
    if call.options.flag("clear") {
        set_ci_root(&home, &project, None).map_err(|e| failed(&e))?;
        call.inv.say(&[&format!(
            "ci-root of {project} cleared: the project root is used again."
        )]);
        return Ok(());
    }
    let dir = call
        .positional
        .get(2)
        .ok_or_else(|| CoordError::refuse(CI_ROOT_USAGE))?;
    let full = resolve(&call.inv.cwd, Path::new(dir));
    if !full.exists() {
        return Err(CoordError::refuse(format!(
            "{} does not exist.",
            full.display()
        )));
    }
    let kept = slashed(&full);
    set_ci_root(&home, &project, Some(&kept)).map_err(|e| failed(&e))?;
    call.inv.say(&[&format!(
        "ci-root of {project}: {kept} (coord check and coord pr merge run Brock there)."
    )]);
    Ok(())
}

/// `coord remote [status|local|remote|synced|ci-root <project> <dir> | --clear]`.
///
/// # Errors
/// A refusal for an unknown word or a bad root, or a write that fails.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    let word = call.positional.first().cloned();
    match word.as_deref() {
        None | Some("status") => {
            status(call);
            Ok(())
        }
        Some("ci-root") => ci_root(call),
        Some("synced") => synced(call),
        Some(LOCAL) => set(call, LOCAL),
        Some(REMOTE) => set(call, REMOTE),
        Some(_) => Err(CoordError::refuse(USAGE)),
    }
}
