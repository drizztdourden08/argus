//! `coord dashboard <verb> ...`: hands the rest of the line to the dashboard's own command
//! (`argus start|stop|status|open|logs|resync|mcp|wake-stream|install-mcp|autostart ...`), so a
//! project that has only `coord` can reach it. The command runs as a child process of this
//! executable (`ARGUS_EXE` names another one) with the streams of this call.

use std::process::Command;

use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::cli::{Env, Opt, Options, Val};

const USAGE: &str = "coord dashboard <start|stop|status|open [<key>]|logs|resync|mcp|wake-stream|install-mcp|autostart> ...   (see dashboard/README.md)";

/// The option names of a command line, each once, in the order they first appear.
fn option_keys(argv: &[String]) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    let mut i = 0;
    while i < argv.len() {
        let arg = &argv[i];
        i += 1;
        let Some(body) = arg.strip_prefix("--") else {
            continue;
        };
        let (key, valued) = match body.split_once('=') {
            Some((key, _)) => (key, true),
            None => (body, false),
        };
        if !keys.iter().any(|k| k == key) {
            keys.push(key.to_owned());
        }
        if !valued && argv.get(i).is_some_and(|next| !next.starts_with("--")) {
            i += 1;
        }
    }
    keys
}

fn words(key: &str, val: &Val) -> Vec<String> {
    match val {
        Val::Flag => vec![format!("--{key}")],
        Val::Text(text) => vec![format!("--{key}"), text.clone()],
    }
}

/// The words after `dashboard`, as the dashboard command takes them: the positionals, then each
/// option as `--key value`, repeated for a list, bare for a flag.
fn to_argv(positional: &[String], options: &Options, argv: &[String]) -> Vec<String> {
    let mut out = positional.to_vec();
    for key in option_keys(argv) {
        match options.get(&key) {
            Some(Opt::One(val)) => out.extend(words(&key, val)),
            Some(Opt::Many(list)) => out.extend(list.iter().flat_map(|val| words(&key, val))),
            None => {}
        }
    }
    out
}

/// Runs the dashboard command with these words and returns its exit code.
///
/// # Errors
/// The command could not be started.
pub fn run_dashboard(env: &Env, args: &[String]) -> Result<i32, CoordError> {
    let exe = match env.get("ARGUS_EXE") {
        Some(exe) => exe.into(),
        None => std::env::current_exe()
            .map_err(|e| CoordError::refuse(format!("could not run the dashboard: {e}")))?,
    };
    let status = Command::new(exe)
        .args(args)
        .envs(env.iter())
        .status()
        .map_err(|e| CoordError::refuse(format!("could not run the dashboard: {e}")))?;
    Ok(status.code().unwrap_or(1))
}

/// `coord dashboard <verb> ...`.
///
/// # Errors
/// The command could not be started, or ended with a code (the exit code of this call).
pub fn handle(call: &mut Call<'_>) -> Outcome {
    if call.positional.is_empty() {
        call.inv.say(&[USAGE]);
        return Ok(());
    }
    let args = to_argv(&call.positional, &call.options, &call.inv.argv);
    match run_dashboard(&call.inv.env, &args)? {
        0 => Ok(()),
        code => Err(CoordError::Exit(code)),
    }
}
