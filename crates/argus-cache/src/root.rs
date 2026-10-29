//! Where the cache lives: `BROCK_BUILD_CACHE_DIR`, else `~/.brock/cache/build-cache`.

use std::env;
use std::path::{Path, PathBuf};

/// The variable that moves the cache.
pub const ENV_DIR: &str = "BROCK_BUILD_CACHE_DIR";

/// The root for the given variable and home. A relative root is refused: a delete must never depend on the
/// working folder.
pub fn cache_root(env_dir: Option<&str>, home: Option<&Path>) -> Option<PathBuf> {
    let root = match env_dir.map(str::trim).filter(|d| !d.is_empty()) {
        Some(dir) => PathBuf::from(dir),
        None => home?.join(".brock").join("cache").join("build-cache"),
    };
    root.is_absolute().then_some(root)
}

/// The root of this machine's user.
pub fn default_root() -> Option<PathBuf> {
    let home = env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .filter(|h| !h.is_empty())
        .map(PathBuf::from);
    cache_root(env::var(ENV_DIR).ok().as_deref(), home.as_deref())
}
