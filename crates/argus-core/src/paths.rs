//! The data folders: `COORD_HOME` holds the boards, the dashboard home holds Argus's own files.

use std::env;
use std::path::{Path, PathBuf};

/// Every path Argus uses, resolved once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    /// The boards' folder (`COORD_HOME`, default `~/.coord`).
    pub coord_home: PathBuf,
    /// The dashboard home (`DASHBOARD_HOME`, default `<coord_home>/dashboard`).
    pub home: PathBuf,
    /// `config.json`.
    pub config: PathBuf,
    /// `dashboard.db`.
    pub db: PathBuf,
    /// `logs/`.
    pub logs: PathBuf,
    /// `attachments/`.
    pub attachments: PathBuf,
    /// `server.pid`.
    pub pid_file: PathBuf,
}

impl Paths {
    /// Paths under `coord_home`, with the dashboard home given or under it.
    #[must_use]
    pub fn new(coord_home: impl Into<PathBuf>, home: Option<PathBuf>) -> Self {
        let coord_home = coord_home.into();
        let home = home.unwrap_or_else(|| coord_home.join("dashboard"));
        Paths {
            config: home.join("config.json"),
            db: home.join("dashboard.db"),
            logs: home.join("logs"),
            attachments: home.join("attachments"),
            pid_file: home.join("server.pid"),
            coord_home,
            home,
        }
    }

    /// From `COORD_HOME` and `DASHBOARD_HOME`, as the Node server reads them.
    #[must_use]
    pub fn from_env() -> Self {
        let coord_home = non_empty("COORD_HOME").unwrap_or_else(|| home_dir().join(".coord"));
        Self::new(
            absolute(&coord_home),
            non_empty("DASHBOARD_HOME").map(|h| absolute(&h)),
        )
    }

    /// The board folder of a project or the portfolio.
    #[must_use]
    pub fn board_dir(&self, key: &str) -> PathBuf {
        self.coord_home.join(key)
    }
}

fn non_empty(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

fn home_dir() -> PathBuf {
    non_empty("USERPROFILE")
        .or_else(|| non_empty("HOME"))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn absolute(p: &Path) -> PathBuf {
    std::path::absolute(p).unwrap_or_else(|_| p.to_path_buf())
}
