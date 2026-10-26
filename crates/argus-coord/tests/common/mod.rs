//! Helpers shared by the coord tests: a temp home with the usual environment, in-process calls
//! with captured streams, and boards written as files. Each test crate uses only part of it.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use argus_coord::cli::{Options, parse_args};
use argus_coord::{Env, Invocation};
use serde_json::{Value, json};
use tempfile::TempDir;

/// What a call printed and returned.
pub struct Ran {
    pub code: i32,
    pub out: String,
    pub err: String,
}

/// A temp `COORD_HOME` and `DASHBOARD_HOME` and the checkouts under it.
pub struct Home {
    pub dir: TempDir,
}

impl Home {
    pub fn new() -> Self {
        let dir = TempDir::new().expect("temp dir");
        fs::create_dir_all(dir.path().join("coord").join("dashboard")).expect("coord home");
        Home { dir }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn coord_home(&self) -> PathBuf {
        self.path().join("coord")
    }

    pub fn dashboard_home(&self) -> PathBuf {
        self.coord_home().join("dashboard")
    }

    /// The environment of a test call: the temp homes, nothing else (no session, no project).
    pub fn env(&self) -> Env {
        let coord = self.coord_home().to_string_lossy().into_owned();
        let dash = self.dashboard_home().to_string_lossy().into_owned();
        Env::of(&[
            ("COORD_HOME", &coord),
            ("DASHBOARD_HOME", &dash),
            ("DASHBOARD_PORT", "0"),
        ])
    }

    /// An environment with one more variable.
    pub fn env_with(&self, name: &str, value: &str) -> Env {
        let mut pairs: Vec<(String, String)> = self
            .env()
            .iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();
        pairs.retain(|(k, _)| k != name);
        pairs.push((name.to_owned(), value.to_owned()));
        let refs: Vec<(&str, &str)> = pairs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        Env::of(&refs)
    }

    /// Writes `<coord>/<key>/board.json` from a JSON value.
    pub fn write_board(&self, key: &str, board: &Value) {
        let dir = self.coord_home().join(key);
        fs::create_dir_all(&dir).expect("board dir");
        let text = serde_json::to_string_pretty(board).expect("board json");
        fs::write(dir.join("board.json"), format!("{text}\n")).expect("board file");
    }

    /// A folder under the temp home.
    pub fn folder(&self, name: &str) -> PathBuf {
        let dir = self.path().join(name);
        fs::create_dir_all(&dir).expect("folder");
        dir
    }
}

/// Runs `coord` in-process in `cwd` with the given environment.
pub fn coord(argv: &[&str], env: Env, cwd: &Path) -> Ran {
    let (mut inv, out, err) = Invocation::captured(argv, env, cwd.to_path_buf(), "");
    let code = argus_coord::run(&mut inv);
    Ran {
        code,
        out: out.text(),
        err: err.text(),
    }
}

/// A thread of a test board: `(id, role, status, sessionId, root)`.
pub type ThreadSpec<'a> = (&'a str, &'a str, &'a str, Option<&'a str>, Option<&'a Path>);

/// A board with these threads.
pub fn board_with(project: &str, threads: &[ThreadSpec<'_>]) -> Value {
    let mut map = serde_json::Map::new();
    for (id, role, status, session, root) in threads {
        map.insert(
            (*id).to_owned(),
            json!({
                "id": id, "role": role, "status": status,
                "sessionId": session, "root": root.map(|r| r.to_string_lossy().into_owned()),
            }),
        );
    }
    json!({ "schema": 2, "project": project, "threads": map, "counters": {}, "dirty": {} })
}

/// The options of a command line, parsed as `coord` does.
pub fn opts(words: &[&str]) -> Options {
    let argv: Vec<String> = words.iter().map(|w| (*w).to_owned()).collect();
    parse_args(&argv).options
}
