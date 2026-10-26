//! Where the two engines are: the Node CLI (`coord.mjs`) and the built `argus` binary. A missing
//! one is a reason to skip, never a failure, so a machine without `ai-config` or without a
//! built binary still passes the suite.

use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// The Node CLI the harness takes as the specification, unless `COORD_NODE_CLI` names another.
const DEFAULT_NODE_CLI: &str = "X:/ai-config/shared/tools/coord/coord.mjs";

/// The two things a differential run spawns.
#[derive(Clone, Debug)]
pub struct Engines {
    /// `coord.mjs`, run as `node <node_cli> ...` with `COORD_ENGINE=node`.
    pub node_cli: PathBuf,
    /// The `argus` binary, run as `argus coord ...`.
    pub argus: PathBuf,
}

impl Engines {
    /// Finds both engines, or says why the run must be skipped.
    ///
    /// The binary is `ARGUS_EXE` when set; else the `argus` next to the test executables
    /// (`target/<profile>/argus`, which `cargo test --workspace` and `cargo build -p argus`
    /// produce). The Node CLI is `COORD_NODE_CLI` or `X:/ai-config/shared/tools/coord/coord.mjs`.
    pub fn locate() -> Result<Engines, String> {
        let node_cli = Engines::locate_node_cli()?;
        let argus = argus_exe()?;
        Ok(Engines { node_cli, argus })
    }

    /// The Node CLI alone: the path of `coord.mjs`, or why it cannot run. For a test of the
    /// harness itself that needs no Argus binary (it then builds `Engines` with any `argus`).
    pub fn locate_node_cli() -> Result<PathBuf, String> {
        if !node_runs() {
            return Err("node is not installed".to_owned());
        }
        let node_cli = env::var_os("COORD_NODE_CLI")
            .map_or_else(|| PathBuf::from(DEFAULT_NODE_CLI), PathBuf::from);
        if node_cli.is_file() {
            Ok(node_cli)
        } else {
            Err(format!(
                "the Node coord CLI is absent ({})",
                node_cli.display()
            ))
        }
    }

    /// The engines, or `None` after printing the reason the test is skipped.
    pub fn or_skip(test: &str) -> Option<Engines> {
        match Engines::locate() {
            Ok(engines) => Some(engines),
            Err(reason) => {
                eprintln!("skipped {test}: {reason}");
                None
            }
        }
    }
}

fn node_runs() -> bool {
    Command::new("node")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn argus_exe() -> Result<PathBuf, String> {
    if let Some(named) = env::var_os("ARGUS_EXE") {
        let path = PathBuf::from(named);
        return if path.is_file() {
            Ok(path)
        } else {
            Err(format!("ARGUS_EXE does not exist ({})", path.display()))
        };
    }
    let beside = env::current_exe().ok().and_then(|exe| {
        Some(
            exe.parent()?
                .parent()?
                .join(format!("argus{}", env::consts::EXE_SUFFIX)),
        )
    });
    match beside {
        Some(path) if path.is_file() => Ok(path),
        _ => Err("no argus binary: set ARGUS_EXE or run `cargo build -p argus`".to_owned()),
    }
}
