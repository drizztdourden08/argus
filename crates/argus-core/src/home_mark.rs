//! The marker both a shadow home and a preview home carry: a boolean key in the `config.json` of a
//! dashboard home. [`crate::shadow_mark`] and [`crate::preview_mark`] give it a key, a port and
//! their own rules; reading and writing the key is the same for both.

use std::fs;
use std::io;
use std::path::Path;

use serde_json::{Map, Value, json};

use crate::Paths;
use crate::config::pretty;

fn read_object(config: &Path) -> Option<Map<String, Value>> {
    let text = fs::read_to_string(config).ok()?;
    match serde_json::from_str::<Value>(text.trim_start_matches('\u{feff}')) {
        Ok(Value::Object(object)) => Some(object),
        _ => None,
    }
}

/// True when the `config.json` at `config` carries `"<key>": true`. A missing or unreadable file
/// is not marked.
pub(crate) fn marked(config: &Path, key: &str) -> bool {
    read_object(config).is_some_and(|c| c.get(key) == Some(&Value::Bool(true)))
}

/// The config to mark: the file's own object, or a fresh one with `port` when the home has none.
/// `None` for a file that is not a JSON object.
fn base(paths: &Paths, port: u16) -> io::Result<Option<Map<String, Value>>> {
    if paths.config.exists() {
        return Ok(read_object(&paths.config));
    }
    fs::create_dir_all(&paths.home)?;
    let mut fresh = Map::new();
    fresh.insert("port".to_owned(), json!(port));
    Ok(Some(fresh))
}

/// Adds `"<key>": true` to the `config.json` of the home, which is created with `port` when the
/// home has none. Answers false and writes nothing for a config that is not a JSON object.
pub(crate) fn mark(paths: &Paths, key: &str, port: u16) -> io::Result<bool> {
    let Some(mut config) = base(paths, port)? else {
        return Ok(false);
    };
    if config.get(key) != Some(&Value::Bool(true)) {
        config.insert(key.to_owned(), Value::Bool(true));
        crate::fs::write_atomic(&paths.config, pretty(&Value::Object(config)))?;
    }
    Ok(true)
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::{mark, marked};
    use crate::Paths;

    fn home(dir: &tempfile::TempDir) -> Paths {
        Paths::new(dir.path().join("coord"), Some(dir.path().join("home")))
    }

    #[test]
    fn a_home_with_no_config_is_marked_with_the_given_port() {
        let dir = tempfile::tempdir().unwrap();
        let p = home(&dir);
        assert!(!marked(&p.config, "flag"));
        assert!(mark(&p, "flag", 4242).unwrap());
        assert!(marked(&p.config, "flag"));
        let text = std::fs::read_to_string(&p.config).unwrap();
        assert!(text.contains("\"port\": 4242"), "{text}");
    }

    #[test]
    fn an_existing_config_keeps_its_keys_and_gains_the_marker() {
        let dir = tempfile::tempdir().unwrap();
        let p = home(&dir);
        std::fs::create_dir_all(&p.home).unwrap();
        std::fs::write(&p.config, r#"{"port":5,"other":1}"#).unwrap();
        assert!(mark(&p, "flag", 4242).unwrap());
        let text = std::fs::read_to_string(&p.config).unwrap();
        assert!(
            text.contains("\"port\": 5") && text.contains("\"other\": 1"),
            "{text}"
        );
        assert!(marked(&p.config, "flag"));
        assert!(mark(&p, "flag", 4242).unwrap(), "marking twice is the same");
    }

    #[test]
    fn a_config_that_is_not_an_object_is_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let p = home(&dir);
        std::fs::create_dir_all(&p.home).unwrap();
        std::fs::write(&p.config, "[1]").unwrap();
        assert!(!mark(&p, "flag", 4242).unwrap());
        assert_eq!(std::fs::read_to_string(&p.config).unwrap(), "[1]");
    }

    #[test]
    fn only_true_counts_as_the_marker() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("config.json");
        for (text, want) in [
            ("{\"flag\":true}", true),
            ("{\"flag\":false}", false),
            ("{\"flag\":\"true\"}", false),
            ("{}", false),
            ("not json", false),
        ] {
            std::fs::write(&file, text).unwrap();
            assert_eq!(marked(&file, "flag"), want, "{text}");
        }
        assert!(!marked(&dir.path().join("absent.json"), "flag"));
    }
}
