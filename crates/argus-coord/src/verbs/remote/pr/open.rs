//! `coord pr open "<title>" --head <branch> [--base main] [--item I-n] [--body text]`.

use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::json;

use super::apply::apply;
use super::root::project_root;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;

/// The commit a branch points at, when git can tell; the branch name is not an option to git.
fn commit_of(root: &Path, head: &str) -> Option<String> {
    if head.starts_with('-') {
        return None;
    }
    let spec = format!("{head}^{{commit}}");
    let out = Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", &spec])
        .current_dir(root)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let sha = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    (out.status.success() && !sha.is_empty()).then_some(sha)
}

fn title_of(call: &Call<'_>) -> Result<String, CoordError> {
    call.positional
        .first()
        .filter(|t| !t.trim().is_empty())
        .cloned()
        .ok_or_else(|| {
            CoordError::refuse("give the title: coord pr open \"<title>\" --head <branch>.")
        })
}

/// `coord pr open`: the pull request of a branch into the base (`main`), with the commit the
/// branch points at now (`--commit <sha>` names it when git cannot).
pub(super) fn open(call: &mut Call<'_>) -> Outcome {
    let title = title_of(call)?;
    let Some(head) = call.options.text("head").map(str::to_owned) else {
        return Err(CoordError::refuse("--head <branch> is required."));
    };
    let commit = call
        .options
        .text("commit")
        .map(str::to_owned)
        .or_else(|| commit_of(&project_root(call), &head));
    let set = json!({
        "title": title, "body": call.options.text("body"), "head": head,
        "base": call.options.text("base"), "headCommit": commit,
        "item": call.options.text("item"),
    });
    let done = apply(call, &json!({ "kind": "pr-open", "set": set }))?;
    let id = done.created.unwrap_or_default();
    let base = call.options.text("base").unwrap_or("main");
    let at = commit.map_or(String::new(), |c| {
        format!(" at {}", c.get(..7).unwrap_or(&c))
    });
    call.inv
        .say(&[&format!("{id} opened: {head} into {base}{at}")]);
    Ok(())
}
