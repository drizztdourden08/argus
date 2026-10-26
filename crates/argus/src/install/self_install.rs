//! `argus self-install [--dir <folder>] [--print]`: copies the running binary to a stable
//! path, so the `exe` key of the dashboard config never points into `target\`, which every
//! build overwrites. The web build is embedded in the binary, so the one file is everything.
//! The verb never edits a config: it prints the path and the `"exe"` line to add.
//!
//! The in-use rule: the copy goes to a temp name next to the target and is renamed over it. A
//! running `argus.exe` cannot be replaced on Windows, so the rename fails, the temp file is
//! removed, the target is left exactly as it was and the verb refuses with a sentence: stop
//! the dashboard with `argus stop`, then run it again. It never leaves a half copy and never
//! stops anything itself. A target that already holds the same bytes is left alone.

use std::fs;
use std::path::{Path, PathBuf};

use super::env::{Env, Io, Platform};

const USAGE: &str = "Usage: argus self-install [--dir <folder>] [--print]
  --dir <folder>  install into this folder (default: %LOCALAPPDATA%\\Programs\\argus)
  --print         print the path and the config line, copy nothing";

/// The folder used when `--dir` is not given.
#[must_use]
pub fn default_dir(env: &Env) -> PathBuf {
    match env.platform {
        Platform::Windows => env
            .local_appdata
            .clone()
            .unwrap_or_else(|| env.home.join("AppData").join("Local"))
            .join("Programs")
            .join("argus"),
        _ => env.home.join(".local").join("bin"),
    }
}

/// The file name of the installed binary.
#[must_use]
pub fn file_name(env: &Env) -> &'static str {
    if env.platform == Platform::Windows {
        "argus.exe"
    } else {
        "argus"
    }
}

/// The line to add to the `config.json` of the dashboard home.
#[must_use]
pub fn config_line(target: &Path) -> String {
    let path = serde_json::to_string(&target.display().to_string()).unwrap_or_default();
    format!("\"exe\": {path}")
}

struct Flags {
    dir: Option<String>,
    print: bool,
    help: bool,
}

fn parse(args: &[String]) -> Result<Flags, String> {
    let mut flags = Flags {
        dir: None,
        print: false,
        help: false,
    };
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--print" => flags.print = true,
            "--help" | "-h" => flags.help = true,
            "--dir" => {
                let dir = rest.next().ok_or("--dir needs a value")?;
                flags.dir = Some(dir.clone());
            }
            other => return Err(format!("unknown argument \"{other}\"")),
        }
    }
    Ok(flags)
}

fn same_file(first: &Path, second: &Path) -> bool {
    matches!((fs::canonicalize(first), fs::canonicalize(second)), (Ok(one), Ok(two)) if one == two)
}

fn same_bytes(first: &Path, second: &Path) -> bool {
    matches!((fs::read(first), fs::read(second)), (Ok(one), Ok(two)) if one == two)
}

/// Copies `source` over `target` atomically, or leaves `target` untouched and says why.
fn replace(source: &Path, target: &Path) -> Result<(), String> {
    let name = target
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let temp = target.with_file_name(format!("{name}.tmp"));
    fs::copy(source, &temp).map_err(|e| format!("the copy to {} failed: {e}", temp.display()))?;
    fs::rename(&temp, target).map_err(|e| {
        let _removed = fs::remove_file(&temp);
        format!(
            "{} cannot be replaced ({e}). It is most likely in use by a running argus: run `argus stop`, then `argus self-install` again. The installed copy was left as it was.",
            target.display()
        )
    })
}

fn report(target: &Path, verb: &str, io: &mut Io<'_>) {
    (io.out)(&format!("{verb} {}", target.display()));
    (io.out)(&format!(
        "Add this line to the config.json of the dashboard home (nothing was edited): {}",
        config_line(target)
    ));
}

/// Runs the verb and answers the exit code: 0 done, 1 a failure, 2 a usage error.
pub fn run(args: &[String], io: &mut Io<'_>) -> u8 {
    let flags = match parse(args) {
        Ok(flags) => flags,
        Err(why) => {
            (io.err)(&format!("argus: {why}\n{USAGE}"));
            return 2;
        }
    };
    if flags.help {
        (io.out)(USAGE);
        return 0;
    }
    let dir = flags
        .dir
        .as_ref()
        .map_or_else(|| default_dir(&io.env), |d| io.env.cwd.join(d));
    let target = dir.join(file_name(&io.env));
    let source = io.env.exe.clone();
    if flags.print {
        report(&target, "would install to", io);
        return 0;
    }
    if same_file(&source, &target) {
        report(&target, "this is the installed copy:", io);
        return 0;
    }
    if let Err(e) = fs::create_dir_all(&dir) {
        (io.err)(&format!("argus: {} cannot be created: {e}", dir.display()));
        return 1;
    }
    if target.exists() && same_bytes(&source, &target) {
        report(&target, "already up to date:", io);
        return 0;
    }
    match replace(&source, &target) {
        Ok(()) => {
            report(&target, "installed", io);
            0
        }
        Err(why) => {
            (io.err)(&format!("argus: {why}"));
            1
        }
    }
}
