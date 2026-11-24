//! What the `coord run` test files share: the seeded board, the argument list and the launch
//! definition.

use std::fs;
use std::path::Path;

use crate::common::Ran;
use crate::remote::{Rig, alpha_with_worker, ok};
use serde_json::Value;

pub const P: [&str; 2] = ["--project", "alpha"];
pub const AS: [&str; 2] = ["--as", "lead"];

pub fn seed(coord_home: &Path, folder: &Path) {
    argus_testkit::write_board(coord_home, "alpha", &alpha_with_worker(folder));
}

/// `coord run <words> --project alpha <extra>`.
pub fn run_argv<'a>(words: &[&'a str], extra: &[&'a str]) -> Vec<&'a str> {
    ["run"]
        .iter()
        .chain(words)
        .chain(&P)
        .chain(extra)
        .copied()
        .collect()
}

pub fn run(rig: &Rig, words: &[&str], extra: &[&str]) -> Ran {
    rig.run(&run_argv(words, extra))
}

/// A launch whose command is a Node script written to the rig folder.
pub fn define(rig: &Rig, name: &str, source: &str) {
    let file = rig.dir.path().join(format!("{name}.cjs"));
    fs::write(&file, source).expect("script");
    let cmd = format!("node \"{}\"", file.display());
    ok(&run(rig, &["define", name, "--cmd", &cmd], &AS));
}

pub async fn run_doc(rig: &Rig, rid: &str) -> Value {
    rig.t.get(&format!("/api/p/alpha/runs/{rid}")).await.json()["run"].clone()
}
