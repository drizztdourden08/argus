//! `argus self-install [--dir <folder>] [--print]`: copies the running binary to a stable
//! path, so the `exe` key of the dashboard config never points into `target\`, which every
//! build overwrites. The web build is embedded in the binary, so the one file is everything.
//! The verb never edits a config: it prints the path and the `"exe"` line to add.
//!
//! The in-use rule: the copy goes to a temp name next to the target and is renamed over it. A
//! running `argus.exe` cannot be replaced on Windows (every open Claude session holds the
//! installed file through its `argus mcp` bridge), but it can be renamed. So when the replace
//! fails, the installed copy is renamed aside to `argus.exe.old-<UTC stamp>` and the new copy
//! takes its place; running processes keep the old image, new starts use the new file. Each run
//! first deletes the earlier `argus.exe.old-*` copies that are no longer in use. When the rename
//! aside fails too, or the new copy cannot be put in place, the folder is left as it was and the
//! verb refuses with a sentence. It never leaves a half copy, never leaves the folder without an
//! `argus.exe` and never stops anything itself. A target that already holds the same bytes is
//! left alone. A restart of the server and the tray picks up the new copy; bridges pick it up as
//! their sessions restart.

use std::fs;
use std::path::{Path, PathBuf};

use super::desktop::stamp;
use super::env::{Env, Io, Platform};
use super::replace::{Replaced, remove_old_copies, replace};

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

fn aside_line(old: &Path) -> String {
    format!(
        "The running copy was in use and was set aside as {}; it is removed on a later self-install.",
        old.display()
    )
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
    remove_old_copies(&dir, file_name(&io.env));
    if target.exists() && same_bytes(&source, &target) {
        report(&target, "already up to date:", io);
        return 0;
    }
    match replace(&source, &target, &stamp()) {
        Ok(done) => {
            if let Replaced::SetAside(old) = done {
                (io.out)(&aside_line(&old));
            }
            report(&target, "installed", io);
            0
        }
        Err(why) => {
            (io.err)(&format!("argus: {why}"));
            1
        }
    }
}
