//! `coord pr merge PR-n`: Brock does the local git merge, this records it.

use std::path::Path;
use std::process::{Command, Stdio};

use argus_board::json::doc;
use serde_json::json;

use super::apply::apply;
use super::need_pr;
use super::root::project_root;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::verbs::remote::support::link::project_of;
use crate::verbs::remote::support::scope::require_board;

/// `brock pr merge --local PR-n`; on Windows through the shell, where Brock is a `.cmd` file.
fn brock(id: &str) -> Command {
    let words = ["brock", "pr", "merge", "--local", id];
    if cfg!(windows) {
        let mut command = Command::new("cmd");
        command.args(["/d", "/c"]).args(words);
        command
    } else {
        let mut command = Command::new(words[0]);
        command.args(&words[1..]);
        command
    }
}

fn last_line(text: &str) -> &str {
    text.lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .map_or("", str::trim)
}

fn is_commit(text: &str) -> bool {
    (7..=40).contains(&text.len()) && text.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Runs Brock's merge in the project folder. Its last output line is the merge commit; when it
/// fails, its own words are the reason.
fn merged_by_brock(call: &Call<'_>, root: &Path, id: &str) -> Result<String, CoordError> {
    let mut command = brock(id);
    command
        .current_dir(root)
        .envs(call.inv.env.iter())
        .stdin(Stdio::null());
    let out = command
        .output()
        .map_err(|e| CoordError::refuse(format!("cannot run brock in {}: {e}", root.display())))?;
    let said = String::from_utf8_lossy(&out.stdout);
    let complaint = String::from_utf8_lossy(&out.stderr);
    if out.status.success() {
        let sha = last_line(&said);
        return if is_commit(sha) {
            Ok(sha.to_ascii_lowercase())
        } else {
            Err(CoordError::refuse(format!(
                "brock merged but did not print a merge commit (it printed \"{sha}\")."
            )))
        };
    }
    let reason = [complaint.trim(), said.trim()]
        .into_iter()
        .find(|t| !t.is_empty())
        .map_or_else(
            || format!("brock pr merge failed ({}).", out.status),
            str::to_owned,
        );
    Err(CoordError::refuse(reason))
}

/// `coord pr merge PR-n`: only an open pull request merges. On Brock's exit 0 the merge commit it
/// printed is recorded and the pull request is merged; otherwise Brock's reason is printed and
/// the exit is 1, with the pull request untouched.
pub(super) fn merge(call: &mut Call<'_>) -> Outcome {
    let id = need_pr(call.positional.first())?;
    let key = project_of(call).to_owned();
    let board = require_board(call, &key)?;
    let Some(pr) = doc(&board, "prs", &id) else {
        return Err(CoordError::refuse(format!("no pull request \"{id}\".")));
    };
    if pr["status"] != "open" {
        let status = pr["status"].as_str().unwrap_or_default();
        return Err(CoordError::refuse(format!(
            "{id} is {status}; only an open pull request merges."
        )));
    }
    let base = pr["base"].as_str().unwrap_or("main").to_owned();
    let commit = merged_by_brock(call, &project_root(call), &id)?;
    let target = json!({ "c": "prs", "id": id });
    let op = json!({ "kind": "pr-merge", "target": target, "set": { "mergeCommit": commit } });
    apply(call, &op)?;
    let short = commit.get(..7).unwrap_or(&commit);
    call.inv
        .say(&[&format!("{id} merged into {base} at {short}")]);
    Ok(())
}
