//! `argus open [<key>] [--print]`: the overview or a board in the browser, the server started
//! first when it is down. `--print` only prints the address.

use argus_core::Paths;
use argus_core::ids::is_key;
use argus_core::js::encode_uri_component;
use argus_native::{TrayFiles, clear_quit_flag};
use serde_json::json;

use super::live::{Up, api, up};
use super::spawn::browser;
use super::start::{Options, START_WAIT, start};

/// The address to open: the plain one, or a one-time sign-in link when the config requires the
/// token.
async fn address(running: &Up, next: &str) -> Result<String, String> {
    let base = running.at.base();
    if running.at.token.is_none() {
        return Ok(format!("{base}{next}"));
    }
    let reply = api(&running.at, "POST", "/api/auth/code", &json!({})).await?;
    let code = reply["code"].as_str().unwrap_or_default();
    Ok(format!(
        "{base}/auth?code={code}&next={}",
        encode_uri_component(next)
    ))
}

/// Opens, or prints, a page of the dashboard.
///
/// # Errors
///
/// None; refusals are printed with their exit code.
pub async fn open(paths: &Paths, key: Option<&str>, print: bool) -> Result<u8, String> {
    if let Some(key) = key.filter(|k| !is_key(k)) {
        eprintln!("dashboard: \"{key}\" is not a project key");
        return Ok(2);
    }
    let _cleared = clear_quit_flag(&TrayFiles::new(&paths.home));
    let found = up(paths).await;
    if let Some(code) = found.as_ref().and_then(|u| u.refuse_node("open")) {
        return Ok(code);
    }
    if found.is_none() {
        let options = Options {
            port: None,
            wait: START_WAIT,
            quiet: true,
        };
        let code = start(paths, &options).await?;
        if code != 0 {
            return Ok(code);
        }
    }
    let Some(running) = up(paths).await else {
        eprintln!("dashboard: the server is not reachable");
        return Ok(1);
    };
    let next = key.map_or_else(|| "/".to_owned(), |k| format!("/p/{k}/"));
    let url = address(&running, &next).await?;
    if print {
        println!("{url}");
        return Ok(0);
    }
    browser(&url).map_err(|e| format!("could not open a browser ({e})"))?;
    println!("opening {}{next}", running.at.base());
    Ok(0)
}
