//! Reading a proof file a tool names by path, only from the folders the owner allows: the
//! project roots, the coord home, the temp folder and `config.attach.extraRoots`.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::ToolError;

fn normal(path: &Path) -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(path.to_string_lossy().to_lowercase())
    } else {
        path.to_path_buf()
    }
}

fn is_inside(real: &Path, roots: &[PathBuf]) -> bool {
    let real = normal(real);
    roots
        .iter()
        .filter_map(|r| fs::canonicalize(r).ok())
        .any(|root| real.starts_with(normal(&root)))
}

fn outside_roots() -> ToolError {
    ToolError::new(
        "forbidden",
        "path is outside the allowed folders (project roots, the coord home, the temp folder, config attach.extraRoots)",
    )
}

/// The bytes of `path` when it is absolute, exists, lies inside one of `roots` or the temp
/// folder (links are followed first) and is at most `max_bytes` long.
pub fn read_allowed(path: &str, max_bytes: u64, roots: &[PathBuf]) -> Result<Vec<u8>, ToolError> {
    let given = Path::new(path);
    if !given.is_absolute() {
        return Err(ToolError::bad_request("path must be absolute"));
    }
    let real =
        fs::canonicalize(given).map_err(|_| ToolError::not_found(format!("no file at {path}")))?;
    let mut all = roots.to_vec();
    all.push(std::env::temp_dir());
    if !is_inside(&real, &all) {
        return Err(outside_roots());
    }
    let size = fs::metadata(&real).map_or(0, |m| m.len());
    if size > max_bytes {
        return Err(ToolError::new(
            "too_large",
            format!("file is {size} bytes, the limit is {max_bytes}"),
        ));
    }
    fs::read(&real).map_err(|e| ToolError::new("internal", e.to_string()))
}
