//! The shadow marker: `"shadow": true` in the `config.json` of a dashboard home. A home with the
//! marker is a shadow home for good. Every way of starting a server on it (`serve`, `start`,
//! the tray, the MCP bridge, the hook autostart) ends in `serve`, which reads the marker and
//! runs in shadow mode whatever the flags say, so a start that forgot `--shadow` cannot write
//! the boards of the real `COORD_HOME`. There is no flag that turns the marker off: the owner
//! deletes the shadow home.

use std::fs;
use std::io;
use std::path::Path;

use serde_json::{Map, Value, json};

use crate::Paths;
use crate::config::pretty;

/// The key of the marker in `config.json`.
pub const KEY: &str = "shadow";
/// The port of a shadow: the Node server keeps 47615 beside it.
pub const SHADOW_PORT: u16 = 47616;

pub(crate) fn read_object(config: &Path) -> Option<Map<String, Value>> {
    let text = fs::read_to_string(config).ok()?;
    match serde_json::from_str::<Value>(text.trim_start_matches('\u{feff}')) {
        Ok(Value::Object(object)) => Some(object),
        _ => None,
    }
}

/// True when the `config.json` at `config` carries `"shadow": true`. A missing or unreadable
/// file is not a shadow.
#[must_use]
pub fn marked(config: &Path) -> bool {
    read_object(config).is_some_and(|c| c.get(KEY) == Some(&Value::Bool(true)))
}

/// True when the home is the default home under `COORD_HOME`: the live dashboard's own folder,
/// which is never marked.
fn is_live_home(paths: &Paths) -> bool {
    Paths::new(paths.coord_home.clone(), None).home == paths.home
}

/// The config to mark: the file's own object, or a fresh one with the port of the shadow when
/// the home has none. `None` for a file that is not a JSON object.
fn base(paths: &Paths) -> io::Result<Option<Map<String, Value>>> {
    if paths.config.exists() {
        return Ok(read_object(&paths.config));
    }
    fs::create_dir_all(&paths.home)?;
    let mut fresh = Map::new();
    fresh.insert("port".to_owned(), json!(SHADOW_PORT));
    Ok(Some(fresh))
}

/// Marks the home as a shadow: the key is added to its `config.json`, which is created with the
/// port of the shadow when the home has none. Does nothing, and answers false, for the default
/// home under `COORD_HOME` (the live dashboard) and for a config that is not a JSON object.
///
/// # Errors
///
/// The folder or the file could not be written.
pub fn mark(paths: &Paths) -> io::Result<bool> {
    if is_live_home(paths) {
        return Ok(false);
    }
    let Some(mut config) = base(paths)? else {
        return Ok(false);
    };
    if config.get(KEY) != Some(&Value::Bool(true)) {
        config.insert(KEY.to_owned(), Value::Bool(true));
        crate::fs::write_atomic(&paths.config, pretty(&Value::Object(config)))?;
    }
    Ok(true)
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::{SHADOW_PORT, mark, marked};
    use crate::Paths;

    fn paths(dir: &tempfile::TempDir, home: &str) -> Paths {
        Paths::new(dir.path().join("coord"), Some(dir.path().join(home)))
    }

    #[test]
    fn a_home_with_no_config_is_marked_with_the_port_of_the_shadow() {
        let dir = tempfile::tempdir().unwrap();
        let p = paths(&dir, "shadow");
        assert!(!marked(&p.config));
        assert!(mark(&p).unwrap());
        assert!(marked(&p.config));
        let text = std::fs::read_to_string(&p.config).unwrap();
        assert!(text.contains(&format!("\"port\": {SHADOW_PORT}")), "{text}");
    }

    #[test]
    fn an_existing_config_keeps_its_keys_and_gains_the_marker() {
        let dir = tempfile::tempdir().unwrap();
        let p = paths(&dir, "shadow");
        std::fs::create_dir_all(&p.home).unwrap();
        std::fs::write(&p.config, r#"{"port":5,"other":1}"#).unwrap();
        assert!(mark(&p).unwrap());
        let text = std::fs::read_to_string(&p.config).unwrap();
        assert!(
            text.contains("\"port\": 5") && text.contains("\"other\": 1"),
            "{text}"
        );
        assert!(marked(&p.config));
        assert!(mark(&p).unwrap(), "marking twice is the same");
    }

    #[test]
    fn the_live_home_and_a_config_that_is_not_an_object_are_never_marked() {
        let dir = tempfile::tempdir().unwrap();
        let live = Paths::new(dir.path().join("coord"), None);
        assert!(!mark(&live).unwrap());
        assert!(!live.config.exists());
        let p = paths(&dir, "shadow");
        std::fs::create_dir_all(&p.home).unwrap();
        std::fs::write(&p.config, "[1]").unwrap();
        assert!(!mark(&p).unwrap());
        assert_eq!(std::fs::read_to_string(&p.config).unwrap(), "[1]");
    }

    #[test]
    fn only_true_counts_as_the_marker() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("config.json");
        for (text, want) in [
            ("{\"shadow\":true}", true),
            ("{\"shadow\":false}", false),
            ("{\"shadow\":\"true\"}", false),
            ("{}", false),
            ("not json", false),
        ] {
            std::fs::write(&file, text).unwrap();
            assert_eq!(marked(&file), want, "{text}");
        }
        assert!(!marked(&dir.path().join("absent.json")));
    }
}
