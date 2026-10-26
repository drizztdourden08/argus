//! `argus install-mcp [--desktop [--write] [--config <file>]] [--code [--dir <folder>]
//! [--project <key>]]`. `--desktop` prints the Claude Desktop entry and writes nothing unless
//! `--write` is given, and then only the `coord-dashboard` key changes. `--code` prints the
//! `.mcp.json` entry for a folder ai-config does not render.

use std::path::PathBuf;

use super::code::{code_entry, resolve_key};
use super::desktop::{SERVER_NAME, desktop_config_path, desktop_entry, write_desktop_config};
use super::env::Io;

const USAGE: &str = "Usage: argus install-mcp [--desktop [--write]] [--code [--dir <project dir>] [--project <key>]]
  --desktop          print the Claude Desktop entry (writes nothing)
  --desktop --write  merge it into claude_desktop_config.json (backup first, only the coord-dashboard key)
  --code             print the .mcp.json entry for a folder ai-config does not render (default: the current folder)";

#[derive(Default)]
struct Flags {
    desktop: bool,
    write: bool,
    code: bool,
    dir: Option<String>,
    config: Option<String>,
    project: Option<String>,
}

fn value(args: &mut std::slice::Iter<'_, String>, flag: &str) -> Result<String, String> {
    args.next()
        .cloned()
        .ok_or_else(|| format!("{flag} needs a value"))
}

/// The flags, and whether help was asked for.
fn parse(args: &[String]) -> Result<(Flags, bool), String> {
    let mut flags = Flags::default();
    let mut help = false;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--desktop" => flags.desktop = true,
            "--write" => flags.write = true,
            "--code" => flags.code = true,
            "--help" | "-h" => help = true,
            "--dir" => flags.dir = Some(value(&mut rest, arg)?),
            "--config" => flags.config = Some(value(&mut rest, arg)?),
            "--project" => flags.project = Some(value(&mut rest, arg)?),
            other => return Err(format!("unknown argument \"{other}\"")),
        }
    }
    Ok((flags, help))
}

fn desktop(flags: &Flags, io: &mut Io<'_>) -> u8 {
    let entry = desktop_entry(&io.env.exe);
    let target = flags
        .config
        .as_ref()
        .map_or_else(|| desktop_config_path(&io.env), |c| io.env.cwd.join(c));
    if !flags.write {
        (io.out)(&serde_json::to_string_pretty(&entry).unwrap_or_default());
        (io.err)(&format!(
            "Not written. Add it to {} yourself, or run: argus install-mcp --desktop --write",
            target.display()
        ));
        return 0;
    }
    match write_desktop_config(&target, &entry) {
        Ok(done) => {
            let kept = done.backup.map_or_else(
                || " (new file)".to_owned(),
                |b| format!(" (backup: {})", b.display()),
            );
            (io.out)(&format!(
                "{SERVER_NAME} written to {}{kept}. Restart Claude Desktop.",
                done.file.display()
            ));
            0
        }
        Err(why) => {
            (io.err)(&format!("argus: {why}"));
            1
        }
    }
}

fn code(flags: &Flags, io: &mut Io<'_>) -> u8 {
    let dir: PathBuf = match &flags.dir {
        Some(given) => io.env.cwd.join(given),
        None => io.env.cwd.clone(),
    };
    let Some(key) = resolve_key(flags.project.as_deref(), &dir) else {
        (io.err)(&format!(
            "argus: no project key for {}: pass --project <key> (a folder ai-config renders already has it in .mcp.json)",
            dir.display()
        ));
        return 1;
    };
    (io.err)(&format!("Entry for {}:", dir.join(".mcp.json").display()));
    let entry = code_entry(&key, io.env.platform);
    (io.out)(&serde_json::to_string_pretty(&entry).unwrap_or_default());
    0
}

fn usage_error(io: &mut Io<'_>, why: &str) -> u8 {
    (io.err)(&format!("argus: {why}\n{USAGE}"));
    2
}

/// Runs the verb and answers the exit code: 0 done, 1 a failure, 2 a usage error.
pub fn run(args: &[String], io: &mut Io<'_>) -> u8 {
    let (flags, help) = match parse(args) {
        Ok(parsed) => parsed,
        Err(why) => return usage_error(io, &why),
    };
    if help || (!flags.desktop && !flags.code) {
        (io.out)(USAGE);
        return if help { 0 } else { 2 };
    }
    if flags.write && !flags.desktop {
        return usage_error(io, "--write goes with --desktop");
    }
    let desktop_done = if flags.desktop {
        desktop(&flags, io)
    } else {
        0
    };
    if desktop_done != 0 || !flags.code {
        return desktop_done;
    }
    code(&flags, io)
}
