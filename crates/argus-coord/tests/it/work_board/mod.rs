//! A seeded `alpha` board for the work verb tests: a coordinator, a worker and one item (I-1), in
//! a temp home, and calls of `coord` as the worker or the coordinator. Each test binary uses only
//! part of it.
#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

use crate::common::{Home, Ran, coord};
use crate::support::differ::FixtureBoard;

/// A temp home with the seeded board and the folder the calls run in.
pub struct Work {
    pub home: Home,
    pub cwd: PathBuf,
}

impl Work {
    pub fn new() -> Self {
        let home = Home::new();
        let cwd = home.folder("work");
        let board = FixtureBoard::new("alpha")
            .thread("coordinator", "coordinator", None)
            .thread("worker", "thread", None)
            .item("Login", "worker", "todo", &[]);
        let dir = home.coord_home().join("alpha");
        fs::create_dir_all(&dir).expect("board folder");
        let text = board.to_json(&cwd.to_string_lossy());
        fs::write(dir.join("board.json"), text).expect("board file");
        Work { home, cwd }
    }

    /// `coord <words> --project alpha --as <who>`.
    pub fn as_thread(&self, who: &str, words: &[&str]) -> Ran {
        let mut argv: Vec<&str> = words.to_vec();
        argv.extend(["--project", "alpha", "--as", who]);
        coord(&argv, self.home.env(), &self.cwd)
    }

    pub fn worker(&self, words: &[&str]) -> Ran {
        self.as_thread("worker", words)
    }

    pub fn lead(&self, words: &[&str]) -> Ran {
        self.as_thread("coordinator", words)
    }

    /// The board file as JSON.
    pub fn board(&self) -> Value {
        let text = fs::read_to_string(self.home.coord_home().join("alpha").join("board.json"))
            .expect("board file");
        serde_json::from_str(&text).expect("board json")
    }

    /// Writes a changed board back, as `store.mjs` does.
    pub fn save(&self, board: &Value) {
        let text = serde_json::to_string_pretty(board).expect("board json");
        let file = self.home.coord_home().join("alpha").join("board.json");
        fs::write(
            file,
            format!(
                "{text}
"
            ),
        )
        .expect("board file");
    }
}
