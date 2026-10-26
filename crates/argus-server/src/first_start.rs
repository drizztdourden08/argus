//! The first start of Argus on a home the Node dashboard used: before anything opens the
//! database for writing, a `VACUUM INTO` copy named `dashboard.db.node-<UTC stamp>` is taken
//! beside it, then a marker file records that Argus has been there. The marker is what tells a
//! home last opened by Node from one Argus already owns.

use std::path::PathBuf;

use argus_core::Paths;
use argus_core::clock::file_stamp;
use argus_store::snapshot;

use crate::ServeError;

/// The file that says Argus has opened this home.
pub const MARKER_FILE: &str = "argus.marker";

/// The backup of the Node database, taken once, on the first start of Argus on a home. A home
/// with no database has nothing to keep; it only gets the marker. Returns the backup's path when
/// one was made.
///
/// # Errors
/// The backup or the marker could not be written; the server must not start without the copy.
pub fn back_up_node_database(paths: &Paths) -> Result<Option<PathBuf>, ServeError> {
    let marker = paths.home.join(MARKER_FILE);
    if marker.exists() {
        return Ok(None);
    }
    let backup = paths.db.is_file().then(|| {
        let mut name = paths.db.clone().into_os_string();
        name.push(format!(".node-{}", file_stamp()));
        PathBuf::from(name)
    });
    if let Some(to) = &backup {
        snapshot(&paths.db, to)?;
    }
    argus_core::fs::write_atomic(&marker, "argus has opened this home\n")?;
    Ok(backup)
}
