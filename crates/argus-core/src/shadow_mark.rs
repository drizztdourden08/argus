//! The shadow marker: `"shadow": true` in the `config.json` of a dashboard home. A home with the
//! marker is a shadow home for good. Every way of starting a server on it (`serve`, `start`,
//! the tray, the MCP bridge, the hook autostart) ends in `serve`, which reads the marker and
//! runs in shadow mode whatever the flags say, so a start that forgot `--shadow` cannot write
//! the boards of the real `COORD_HOME`. There is no flag that turns the marker off: the owner
//! deletes the shadow home.

use std::io;
use std::path::Path;

use crate::Paths;
use crate::home_mark;

/// The key of the marker in `config.json`.
pub const KEY: &str = "shadow";
/// The port of a shadow: the Node server keeps 47615 beside it.
pub const SHADOW_PORT: u16 = 47616;

/// True when the `config.json` at `config` carries `"shadow": true`. A missing or unreadable
/// file is not a shadow.
#[must_use]
pub fn marked(config: &Path) -> bool {
    home_mark::marked(config, KEY)
}

/// True when the home is the default home under `COORD_HOME`: the live dashboard's own folder,
/// which is never marked.
fn is_live_home(paths: &Paths) -> bool {
    Paths::new(paths.coord_home.clone(), None).home == paths.home
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
    home_mark::mark(paths, KEY, SHADOW_PORT)
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::{SHADOW_PORT, mark, marked};
    use crate::Paths;

    #[test]
    fn a_shadow_home_is_marked_with_the_shadow_key_and_port() {
        let dir = tempfile::tempdir().unwrap();
        let p = Paths::new(dir.path().join("coord"), Some(dir.path().join("shadow")));
        assert!(mark(&p).unwrap());
        assert!(marked(&p.config));
        let text = std::fs::read_to_string(&p.config).unwrap();
        assert!(text.contains(&format!("\"port\": {SHADOW_PORT}")), "{text}");
    }

    #[test]
    fn the_live_home_is_never_marked() {
        let dir = tempfile::tempdir().unwrap();
        let live = Paths::new(dir.path().join("coord"), None);
        assert!(!mark(&live).unwrap());
        assert!(!live.config.exists());
    }
}
