//! A snapshot of a home: every file under its root (the board tree, the dashboard folder and
//! the work folder, where `launch.json` and friends land), normalised.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use super::normalise::Normaliser;

/// Relative path (normalised, `/` separated) to normalised content.
pub type Snapshot = BTreeMap<String, String>;

/// Reads every file under `root`.
pub fn snapshot(root: &Path, norm: &Normaliser) -> io::Result<Snapshot> {
    let mut files = Snapshot::new();
    walk(root, root, norm, &mut files)?;
    Ok(files)
}

fn walk(root: &Path, dir: &Path, norm: &Normaliser, files: &mut Snapshot) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            walk(root, &path, norm, files)?;
            continue;
        }
        let relative = path.strip_prefix(root).expect("under the root");
        let name = relative.to_string_lossy().replace('\\', "/");
        files.insert(norm.text(&name), norm.bytes(&fs::read(&path)?));
    }
    Ok(())
}
