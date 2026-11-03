//! `coord launcher set <name> --cmd "<command>" [flags]`: make the launcher, or change the flags
//! given. `--cmd` is required for a new one and never empty; a retired one comes back. A
//! standalone launcher (not master, not made for a bundle) must have `--use-every`. A new build
//! needs `--for I-n`, the item it serves (it closes with that item); `--master` makes it the
//! master build instead, which needs none and takes the role from the build that had it.

use argus_board::Board;
use argus_board::json::{doc, truthy};
use argus_launchers::launcher::{
    Launcher, LauncherPatch, MASTER, Platform, WriteContext, WriteResult, command_warnings,
    has_master, runs_note, write_launcher,
};

use super::name::name_arg;
use super::patch::patch_from;
use super::report::{handed_over, ignored_every};
use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};
use crate::verbs::agent::base::links::linked;
use crate::verbs::agent::base::serves::check_link;
use crate::verbs::agent::base::{Run, write};
use crate::verbs::agent::rules::{self, typed_board};

/// `"Title", dev, current, branch b, used every 7d, command line: ...`: what a new launcher is.
fn describe(rec: &Launcher) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(title) = rec.title.as_deref().filter(|t| !t.is_empty()) {
        parts.push(format!("\"{title}\""));
    }
    if rec.kind != "app" {
        parts.push(rec.kind.clone());
    }
    if rec.current {
        parts.push("current".to_owned());
    }
    if rec.master {
        parts.push("master build".to_owned());
    }
    if let Some(item) = rec.for_item.as_deref() {
        parts.push(format!("serves {item}"));
    }
    if let Some(branch) = rec.branch.as_deref().filter(|b| !b.is_empty()) {
        parts.push(format!("branch {branch}"));
    }
    if let Some(every) = rec.use_every.as_deref().filter(|e| !e.is_empty()) {
        parts.push(format!("used every {every}"));
    }
    parts.push(format!(
        "command line: {}",
        rec.cmd.as_deref().unwrap_or_default()
    ));
    parts.join(", ")
}

fn what_happened(name: &str, written: &WriteResult, rec: &Launcher) -> String {
    let changed = written.changed.join(", ");
    if written.created {
        format!("{name}: launcher added ({}).", describe(rec))
    } else if written.revived && written.changed.is_empty() {
        format!("{name}: brought back from retired.")
    } else if written.revived {
        format!("{name}: brought back from retired and updated ({changed}).")
    } else if written.changed.is_empty() {
        format!("{name}: unchanged.")
    } else {
        format!("{name}: updated ({changed}).")
    }
}

/// The notes after the first line: an ignored `--use-every`, the command line and what runs it,
/// the warnings, and the missing master.
fn notes(
    run: &mut Run<'_>,
    board: &Board,
    name: &str,
    written: &WriteResult,
    rec: &Launcher,
) -> Result<(), CoordError> {
    if written.ignored.contains(&"useEvery") {
        run.say(&ignored_every(name, rec));
    }
    if let Some(old) = written.previous_master.as_deref() {
        run.say(&handed_over(name, old));
    }
    if written.created || written.changed.contains(&"cmd") {
        let cmd = rec.cmd.as_deref().unwrap_or_default();
        run.say(&format!("command line: {cmd}"));
        run.say(&format!(
            "{} One line only: chain with && (not ; and not a newline), use double quotes, set the folder with --cwd.",
            runs_note(Platform::current())
        ));
        for warning in command_warnings(cmd, Platform::current()) {
            run.say(&format!("warning: {warning}"));
        }
    }
    if name != MASTER && !has_master(&typed_board(board)?) {
        run.say(&format!(
            "warning: this project has no master launcher and needs one (the main checkout's app): coord launcher set {MASTER} --cmd \"<command>\". Until then coord build hand is refused."
        ));
    }
    Ok(())
}

/// `coord launcher set`.
///
/// # Errors
/// A flag the rules refuse, nothing to change, a launcher of another thread, a Play name that is
/// taken.
pub fn run(call: &mut Call<'_>) -> Outcome {
    let name = name_arg(&call.positional, "set")?;
    let patch = patch_from(&call.options)?;
    write(call, |board, who, run| {
        let have = doc(board, "launchers", &name);
        let live = have.is_some_and(|h| !truthy(h, "retiredAt"));
        if live && patch == LauncherPatch::default() {
            return Err(CoordError::refuse(
                "nothing to change: pass --cmd --title --cwd --branch --commit --note --kind --current --window --use-every --for or --master (\"\" clears a text field).",
            ));
        }
        check_link(board, &name, &patch)?;
        let cwd = run.cwd.to_string_lossy().into_owned();
        let written = rules::run(board, who, |b, w, at| {
            write_launcher(
                b,
                w,
                &name,
                &patch,
                &WriteContext {
                    cwd: &cwd,
                    bundle: None,
                    at,
                },
            )
        })?;
        let typed = typed_board(board)?;
        let rec = typed.launchers.get(&name).cloned().unwrap_or_default();
        let links = linked(board, &name, None);
        let play = if written.play.is_empty() {
            String::new()
        } else {
            format!(" {}", written.play)
        };
        let by = if links.is_empty() {
            String::new()
        } else {
            format!(" Linked by bundle {}.", links.join(", "))
        };
        run.say(&format!(
            "{}{play}{by}",
            what_happened(&name, &written, &rec)
        ));
        notes(run, board, &name, &written, &rec)
    })
}
