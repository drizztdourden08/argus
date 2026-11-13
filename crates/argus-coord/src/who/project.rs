//! Which project a call is for, most reliable source first:
//!
//! 1. `--project`
//! 2. `COORD_PROJECT`
//! 3. the board that has a live thread holding this session id
//! 4. the `projectKey` of the nearest `.ai-config.json`
//! 5. a `.coord-project` marker

use std::fs;
use std::path::Path;

use argus_core::js;
use serde_json::Value;

use super::board_ref::board_ref;
use super::path::holder_of;
use super::session::project_of_session;
use crate::cli::Options;
use crate::cli::env::Env;
use crate::cli::exit::CoordError;

const UNKNOWN: &str =
    "cannot tell which project this is: no .ai-config.json above the cwd. Pass --project <key>.";

fn read_json(file: &Path) -> Option<Value> {
    serde_json::from_str(
        fs::read_to_string(file)
            .ok()?
            .trim_start_matches('\u{feff}'),
    )
    .ok()
}

fn wired_key(cwd: &Path) -> Option<String> {
    let dir = holder_of(cwd, ".ai-config.json")?;
    let key = read_json(&dir.join(".ai-config.json"))?["projectKey"].clone();
    js::truthy(&key).then(|| js::string(&key))
}

fn marked_key(cwd: &Path) -> Result<Option<String>, CoordError> {
    let Some(dir) = holder_of(cwd, ".coord-project") else {
        return Ok(None);
    };
    let text = fs::read_to_string(dir.join(".coord-project"))?;
    Ok(Some(js::trim(&text).to_owned()))
}

fn named(
    env: &Env,
    cwd: &Path,
    options: &Options,
    sid: Option<&str>,
    home: &Path,
) -> Result<Option<String>, CoordError> {
    if let Some(project) = options.text("project") {
        return Ok(Some(project.to_owned()));
    }
    if let Some(project) = env.get("COORD_PROJECT") {
        return Ok(Some(project.to_owned()));
    }
    let sid = sid.filter(|s| !s.is_empty()).or_else(|| env.session_id());
    if let Some(project) = sid.and_then(|s| project_of_session(home, s)) {
        return Ok(Some(project));
    }
    match wired_key(cwd) {
        Some(key) => Ok(Some(key)),
        None => marked_key(cwd),
    }
}

/// The board id of a call. `sid` is the session id (a hook passes the one of its input, else
/// the environment's). The first source that names a project wins; its value is an id, an old
/// key or a legacy folder, and the result is the id.
///
/// # Errors
/// Refused when no source names a project, or the name is no board of a migrated home.
pub fn find_project(
    env: &Env,
    cwd: &Path,
    options: &Options,
    sid: Option<&str>,
) -> Result<String, CoordError> {
    let home = env.paths(cwd).coord_home;
    let value = named(env, cwd, options, sid, &home)?.ok_or_else(|| CoordError::refuse(UNKNOWN))?;
    board_ref(&home, &value)
}
