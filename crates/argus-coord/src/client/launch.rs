//! How the CLI brings the dashboard up, as data first: a [`Plan`] is a program, its arguments and
//! the variables it needs, so a test checks the plan without starting anything and without an
//! icon. The client autostart and the session hooks share it.
//!
//! Both start the dashboard through the tray on Windows, as the Node hooks do (`dashboard tray
//! start` shows the icon and starts the server), and plain `start` elsewhere.
//!
//! - A home that carries the shadow marker (`"shadow": true` in its `config.json`) is started
//!   with the Argus binary its config names (`exe`), through `argus tray start --shadow`. It is
//!   never started with the Node bin, and with no `exe` to run nothing is started.
//! - Any other home is started with the Node bin of the install, as before the switch-over
//!   (after it, that bin forwards to Argus).

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use argus_core::Paths;
use argus_core::launch;
use argus_core::shadow_mark;

use super::config::{dashboard_config, read_json};
use crate::cli::env::Env;

/// `DETACHED_PROCESS | CREATE_NO_WINDOW`: no console, and not tied to this process.
#[cfg(windows)]
const HIDDEN: u32 = 0x0800_0008;

/// A program to start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// The program.
    pub program: PathBuf,
    /// Its arguments.
    pub args: Vec<OsString>,
    /// Variables set for it on top of the caller's environment.
    pub vars: Vec<(String, String)>,
}

/// The install the CLI config of this machine names (`aiConfigDir/dashboard`).
fn installed(env: &Env) -> Option<PathBuf> {
    let cli = read_json(&env.home_dir().join(".rotp-cli").join("config.json"))?;
    Some(Path::new(cli["aiConfigDir"].as_str()?).join("dashboard"))
}

/// `bin/dashboard.mjs` of the install: `COORD_DASHBOARD_INSTALL`, else `config.json`'s
/// `installDir`, else the install of the CLI config; `None` when it is not there.
fn dashboard_bin(env: &Env, paths: &Paths) -> Option<PathBuf> {
    let install = env
        .get("COORD_DASHBOARD_INSTALL")
        .map(PathBuf::from)
        .or_else(|| dashboard_config(paths)?.install_dir.map(PathBuf::from))
        .or_else(|| installed(env))?;
    Some(install.join("bin").join("dashboard.mjs")).filter(|bin| bin.exists())
}

/// `node <bin> tray start` (`start` off Windows).
fn node_plan(bin: &Path, windows: bool) -> Plan {
    let verbs: &[&str] = if windows {
        &["tray", "start"]
    } else {
        &["start"]
    };
    let mut args = vec![bin.as_os_str().to_owned()];
    args.extend(verbs.iter().map(OsString::from));
    Plan {
        program: PathBuf::from("node"),
        args,
        vars: Vec::new(),
    }
}

/// `argus tray start --data-dir <home> --shadow` (`argus start` off Windows, with the home in
/// `DASHBOARD_HOME`).
fn argus_plan(exe: PathBuf, home: &Path, windows: bool) -> Plan {
    let args = if windows {
        launch::tray_start(home)
    } else {
        vec![OsString::from("start")]
    };
    Plan {
        program: exe,
        args,
        vars: vec![("DASHBOARD_HOME".to_owned(), home.display().to_string())],
    }
}

/// The Argus binary the config of a shadow home names, when it is there.
fn shadow_exe(paths: &Paths) -> Option<PathBuf> {
    let exe = read_json(&paths.config)?["exe"]
        .as_str()
        .map(PathBuf::from)?;
    exe.is_file().then_some(exe)
}

/// What to run to bring the dashboard of these paths up; `None` when there is nothing to run
/// (no install found, or a shadow home with no Argus binary named).
#[must_use]
pub fn plan_for(env: &Env, paths: &Paths, windows: bool) -> Option<Plan> {
    if shadow_mark::marked(&paths.config) {
        return shadow_exe(paths).map(|exe| argus_plan(exe, &paths.home, windows));
    }
    dashboard_bin(env, paths).map(|bin| node_plan(&bin, windows))
}

/// [`plan_for`] on this machine.
#[must_use]
pub fn plan(env: &Env, paths: &Paths) -> Option<Plan> {
    plan_for(env, paths, cfg!(windows))
}

/// Starts the plan with no window and not tied to this process, and does not wait. The standard
/// handles of this process stay out of it: on Windows a child inherits every inheritable handle,
/// and the server would hold the pipe the caller reads this call's output from.
pub fn spawn(env: &Env, plan: &Plan) {
    argus_native::keep_stdio_private();
    let mut command = Command::new(&plan.program);
    command
        .args(&plan.args)
        .envs(env.iter())
        .envs(plan.vars.iter().cloned());
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    std::os::windows::process::CommandExt::creation_flags(&mut command, HIDDEN);
    command.spawn().ok();
}
