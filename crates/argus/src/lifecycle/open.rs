//! `argus open [<key>] [--print]`: the overview or a board in the browser, the server started
//! first when it is down. `--print` only prints the address.

use argus_core::Paths;
use argus_core::ids::is_key;
use argus_core::js::encode_uri_component;
use argus_core::names::Door;
use argus_native::{TrayFiles, clear_quit_flag};
use serde_json::{Value, json};

use super::live::{Up, api, up};
use super::spawn::browser;
use super::start::{Options, START_WAIT, start};

/// The base address of a page under the local names, when the server says they are up:
/// `http://<key>.<suffix>`, the overview under `argus` without a key.
fn named_base(health: &Value, key: Option<&str>) -> Option<String> {
    let names = &health["localNames"];
    let door = Door {
        address: names["address"].as_str()?.parse().ok()?,
        suffix: names["suffix"].as_str()?.to_owned(),
        port: u16::try_from(names["doorPort"].as_u64()?).ok()?,
    };
    Some(door.url(key.unwrap_or("argus"), ""))
}

/// The address to open: the plain one, or a one-time sign-in link when the config requires the
/// token.
async fn address(running: &Up, base: &str, next: &str) -> Result<String, String> {
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
    let (base, next) = match named_base(&running.health, key) {
        Some(base) => (base, "/".to_owned()),
        None => (
            running.at.base(),
            key.map_or_else(|| "/".to_owned(), |k| format!("/p/{k}/")),
        ),
    };
    let url = address(&running, &base, &next).await?;
    if print {
        println!("{url}");
        return Ok(0);
    }
    browser(&url).map_err(|e| format!("could not open a browser ({e})"))?;
    println!("opening {base}{next}");
    Ok(0)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::named_base;

    fn health(door_port: u64) -> serde_json::Value {
        json!({ "localNames": { "address": "127.65.82.71", "suffix": "argus", "doorPort": door_port } })
    }

    #[test]
    fn the_names_give_the_board_or_the_overview() {
        let on = health(80);
        assert_eq!(
            named_base(&on, Some("rotp")).as_deref(),
            Some("http://rotp.argus")
        );
        assert_eq!(named_base(&on, None).as_deref(), Some("http://argus.argus"));
    }

    #[test]
    fn a_door_off_port_80_carries_its_port() {
        assert_eq!(
            named_base(&health(47615), Some("rotp")).as_deref(),
            Some("http://rotp.argus:47615")
        );
    }

    #[test]
    fn a_server_without_names_gives_none() {
        assert_eq!(named_base(&json!({ "ok": true }), Some("rotp")), None);
        assert_eq!(
            named_base(&json!({ "localNames": { "suffix": "argus" } }), None),
            None
        );
    }
}
