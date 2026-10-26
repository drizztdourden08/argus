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

/// The project key of a call. `sid` is the session id (a hook passes the one of its input,
/// else the environment's).
///
/// # Errors
/// Refused when no source names a project.
pub fn find_project(
    env: &Env,
    cwd: &Path,
    options: &Options,
    sid: Option<&str>,
) -> Result<String, CoordError> {
    if let Some(project) = options.text("project") {
        return Ok(project.to_owned());
    }
    if let Some(project) = env.get("COORD_PROJECT") {
        return Ok(project.to_owned());
    }
    let sid = sid.filter(|s| !s.is_empty()).or_else(|| env.session_id());
    let home = env.paths(cwd).coord_home;
    if let Some(project) = sid.and_then(|s| project_of_session(&home, s)) {
        return Ok(project);
    }
    if let Some(key) = wired_key(cwd) {
        return Ok(key);
    }
    marked_key(cwd)?.ok_or_else(|| CoordError::refuse(UNKNOWN))
}
