//! The environment of one call: a map of variables instead of the process environment, so a
//! test can run `coord` against temp homes in-process, and the paths and session id read from it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use argus_core::Paths;

use super::resolve::resolve;

/// Environment variables, read by name.
#[derive(Debug, Clone, Default)]
pub struct Env {
    vars: HashMap<String, String>,
}

impl Env {
    /// The environment of this process.
    #[must_use]
    pub fn from_process() -> Self {
        let vars = std::env::vars_os()
            .map(|(k, v)| {
                (
                    k.to_string_lossy().into_owned(),
                    v.to_string_lossy().into_owned(),
                )
            })
            .collect();
        Env { vars }
    }

    /// An environment holding only these variables.
    #[must_use]
    pub fn of(pairs: &[(&str, &str)]) -> Self {
        let vars = pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        Env { vars }
    }

    /// The value of a variable, `None` when unset or empty (JavaScript `process.env.X || ...`).
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.vars
            .get(name)
            .map(String::as_str)
            .filter(|v| !v.is_empty())
    }

    /// Every variable, for a child process.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.vars.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// The home folder of the user (`USERPROFILE`, else `HOME`).
    #[must_use]
    pub fn home_dir(&self) -> PathBuf {
        self.get("USERPROFILE")
            .or_else(|| self.get("HOME"))
            .map_or_else(|| PathBuf::from("."), PathBuf::from)
    }

    /// The Claude Code session id of this call (`CLAUDE_CODE_SESSION_ID`).
    #[must_use]
    pub fn session_id(&self) -> Option<&str> {
        self.get("CLAUDE_CODE_SESSION_ID")
    }

    /// The data folders: `COORD_HOME` (default `~/.coord`) and `DASHBOARD_HOME` (default
    /// `<COORD_HOME>/dashboard`), made absolute against the working directory as Node does.
    #[must_use]
    pub fn paths(&self, cwd: &Path) -> Paths {
        let coord_home = self
            .get("COORD_HOME")
            .map_or_else(|| self.home_dir().join(".coord"), PathBuf::from);
        let home = self
            .get("DASHBOARD_HOME")
            .map(|h| resolve(cwd, Path::new(h)));
        Paths::new(resolve(cwd, &coord_home), home)
    }
}
