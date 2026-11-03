//! The preview marker: `"preview": true` in the `config.json` of a dashboard home. A preview
//! serves a copy of the boards with board writes allowed and every outward effect off (no
//! worktree removal, kill, run, toast or remote call). A home with the marker is a preview home
//! for good, as a shadow home is: every way of starting a server on it ends in `serve`, which
//! reads the marker. A preview never starts on the live boards: [`refusal`] says why.

use std::fs;
use std::io;
use std::path::Path;

use crate::Paths;
use crate::home_mark;

/// The key of the marker in `config.json`.
pub const KEY: &str = "preview";
/// The port of a preview: the live dashboard keeps 47615 and the shadow 47616.
pub const PREVIEW_PORT: u16 = 47617;

/// True when the `config.json` at `config` carries `"preview": true`. A missing or unreadable
/// file is not a preview.
#[must_use]
pub fn marked(config: &Path) -> bool {
    home_mark::marked(config, KEY)
}

fn resolved(path: &Path) -> std::path::PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Why a preview may not start on these paths, or `None`. The boards must not be the live
/// ones (`live`, the default `~/.coord`) and the dashboard home must not lie inside them, so a
/// preview can never write the real boards or share the live dashboard's database and pid file.
#[must_use]
pub fn refusal(paths: &Paths, live: &Path) -> Option<String> {
    let (plain, live) = (live, resolved(live));
    if resolved(&paths.coord_home) == live {
        return Some(format!(
            "a preview never runs on the live boards ({}). Copy them first and set COORD_HOME \
             and DASHBOARD_HOME to the copy.",
            paths.coord_home.display()
        ));
    }
    let inside = paths.home.starts_with(plain) || resolved(&paths.home).starts_with(&live);
    inside.then(|| {
        format!(
            "a preview never runs inside the live home ({}). Give it a copied folder with \
             --data-dir.",
            paths.home.display()
        )
    })
}

/// Marks the home as a preview: the key is added to its `config.json`, which is created with
/// the port of the preview when the home has none. Answers false and writes nothing for a
/// config that is not a JSON object. The caller has checked [`refusal`] first.
///
/// # Errors
///
/// The folder or the file could not be written.
pub fn mark(paths: &Paths) -> io::Result<bool> {
    home_mark::mark(paths, KEY, PREVIEW_PORT)
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::{PREVIEW_PORT, mark, marked, refusal};
    use crate::Paths;

    fn copy_in(dir: &tempfile::TempDir) -> Paths {
        Paths::new(dir.path().join("coord"), Some(dir.path().join("preview")))
    }

    #[test]
    fn a_preview_home_is_marked_with_the_preview_key_and_port() {
        let dir = tempfile::tempdir().unwrap();
        let p = copy_in(&dir);
        assert!(mark(&p).unwrap());
        assert!(marked(&p.config));
        let text = std::fs::read_to_string(&p.config).unwrap();
        assert!(
            text.contains(&format!("\"port\": {PREVIEW_PORT}")),
            "{text}"
        );
    }

    #[test]
    fn the_live_boards_and_a_home_inside_them_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let live = dir.path().join("live");
        std::fs::create_dir_all(live.join("dashboard")).unwrap();
        let on_live = Paths::new(&live, Some(dir.path().join("copy")));
        assert!(refusal(&on_live, &live).unwrap().contains("live boards"));
        assert!(refusal(&Paths::new(&live, None), &live).is_some());
        let inside = Paths::new(dir.path().join("copy"), Some(live.join("dashboard")));
        assert!(refusal(&inside, &live).unwrap().contains("live home"));
    }

    #[test]
    fn a_copy_is_accepted_even_when_its_dashboard_folder_is_the_default_one() {
        let dir = tempfile::tempdir().unwrap();
        let copy = Paths::new(dir.path().join("preview"), None);
        assert_eq!(refusal(&copy, &dir.path().join("live")), None);
    }
}
