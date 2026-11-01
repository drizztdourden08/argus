//! Helpers shared by the WP3 test binaries: a temp home with a fixed tail of options for every
//! call, and the tables of steps of the differential tests.
#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

use crate::common::{Home, Ran, coord};
use crate::support::differ::{LEAD_SESSION, Step, WORKER_SESSION};

/// A temp home and the words every call ends with (the project, the thread).
pub struct Cli {
    pub home: Home,
    pub cwd: PathBuf,
    tail: Vec<&'static str>,
}

impl Cli {
    /// A home whose calls run in `cwd` (a folder under it, or the home itself when empty) and
    /// end with `tail`.
    pub fn new(cwd: &str, tail: &[&'static str]) -> Self {
        let home = Home::new();
        let cwd = if cwd.is_empty() {
            home.path().to_path_buf()
        } else {
            home.folder(cwd)
        };
        Cli {
            home,
            cwd,
            tail: tail.to_vec(),
        }
    }

    pub fn run(&self, words: &[&str]) -> Ran {
        let mut argv = words.to_vec();
        argv.extend(&self.tail);
        coord(&argv, self.home.env(), &self.cwd)
    }

    pub fn ok(&self, words: &[&str]) -> Ran {
        let ran = self.run(words);
        assert_eq!(ran.code, 0, "{words:?}\n{}\n{}", ran.out, ran.err);
        ran
    }

    pub fn refused(&self, words: &[&str], text: &str) {
        let ran = self.run(words);
        assert_eq!(ran.code, 1, "{words:?} should be refused\n{}", ran.out);
        assert!(ran.err.contains(text), "{words:?}\n{}", ran.err);
    }

    pub fn board(&self) -> Value {
        let file = self.home.coord_home().join("alpha").join("board.json");
        serde_json::from_str(&fs::read_to_string(file).expect("board")).expect("json")
    }
}

/// One row of a table of differential steps: who runs it (`l` the lead, `w` the worker, `-`
/// nobody, so `--as` decides) and the words after `coord`.
pub type Row = (&'static str, &'static [&'static str]);

/// The steps of tables of rows; `--project alpha` is added unless the row names a project.
pub fn steps(tables: &[&[Row]]) -> Vec<Step> {
    tables
        .iter()
        .flat_map(|table| table.iter())
        .map(|(who, words)| {
            let mut all: Vec<&str> = words.to_vec();
            if !words.contains(&"--project") {
                all.extend(["--project", "alpha"]);
            }
            let step = Step::new(&all);
            match *who {
                "l" => step.session(LEAD_SESSION),
                "w" => step.session(WORKER_SESSION),
                _ => step,
            }
        })
        .collect()
}
