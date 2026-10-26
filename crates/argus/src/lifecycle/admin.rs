//! `argus resync [--project k] [--hard]` and `argus token --rotate`: the two admin calls.

use argus_core::Paths;
use argus_core::config::{env_port, load_config, new_token, save_config};
use argus_core::ids::is_scope;
use serde_json::{Map, json};

use super::live::{api, up};

const ROTATED: &str = "token rotated (it only matters when config.json has requireToken; then open the dashboard again with: argus open)";

/// Asks the running server to read its boards again and prints its answer.
///
/// # Errors
///
/// The server refused the call.
pub async fn resync(paths: &Paths, project: Option<&str>, hard: bool) -> Result<u8, String> {
    let Some(running) = up(paths).await else {
        eprintln!("dashboard is not running");
        return Ok(1);
    };
    if let Some(code) = running.refuse_node("resync") {
        return Ok(code);
    }
    if let Some(bad) = project.filter(|p| !is_scope(p)) {
        eprintln!("dashboard: \"{bad}\" is not a project key");
        return Ok(2);
    }
    let mut body = Map::new();
    if let Some(project) = project {
        body.insert("project".into(), json!(project));
    }
    body.insert("hard".into(), json!(hard));
    let answer = api(&running.at, "POST", "/api/admin/resync", &body.into()).await?;
    println!("{answer}");
    Ok(0)
}

/// Rotates the token: through the server when it runs, else in `config.json`.
///
/// # Errors
///
/// The server refused the call or the config could not be written.
pub async fn token(paths: &Paths, rotate: bool) -> Result<u8, String> {
    if !rotate {
        eprintln!("Usage: argus token --rotate");
        return Ok(2);
    }
    let found = up(paths).await;
    if let Some(code) = found.as_ref().and_then(|u| u.refuse_node("token")) {
        return Ok(code);
    }
    if let Some(running) = found {
        api(&running.at, "POST", "/api/admin/token/rotate", &json!({})).await?;
    } else {
        let env = env_port().map_err(|e| e.to_string())?;
        let mut config = load_config(paths, env).map_err(|e| e.to_string())?;
        config.token = new_token();
        save_config(paths, &config, env).map_err(|e| e.to_string())?;
    }
    println!("{ROTATED}");
    Ok(0)
}
