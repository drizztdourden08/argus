//! What the seed does besides plain `coord` calls: the other boards and their leads, the
//! requests between boards (the dependency contract of `argus-board`), the merge of one pull
//! request, and the folders of the demo worktrees. All of it lands in the preview home.

use std::fs;

use argus_board::demo::{DepSpec, FOLDERS, INCOMING, MERGE_COMMIT, MERGED_PR, OUTGOING, TARGETS};
use argus_board::deps::{AskRequest, BoardRoster, CloseRequest, Roster, ask, close};
use argus_board::ops::{Role, Who, apply_op};
use argus_board::store::{read_board, with_board};
use serde_json::json;

use super::runner::Runner;

fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|w| (*w).to_owned()).collect()
}

/// Makes the worktree folders and binds their paths, `{root}` and `{wt}` for the steps.
pub(super) fn folders(run: &mut Runner) -> std::io::Result<()> {
    let root = run.home().join(format!("{}-worktrees", run.key()));
    fs::create_dir_all(&root)?;
    let shown = root.to_string_lossy().into_owned();
    run.bind("root", &shown);
    run.bind("wt", &shown);
    for (name, folder) in FOLDERS {
        let path = root.join(folder);
        fs::create_dir_all(&path)?;
        run.bind(name, &path.to_string_lossy());
    }
    Ok(())
}

/// A board and a lead for each board the requests go to, unless the home already has them.
pub(super) fn targets(run: &mut Runner) {
    for target in TARGETS {
        let exists = read_board(run.home(), target).ok().flatten().is_some();
        if !exists {
            let init = words(&["board", "init", "--name", target]);
            let made = run.call(target, "", &init);
            keep(run, target, made);
        }
        if BoardRoster::new(run.home()).lead(target).is_none() {
            let lead = words(&[
                "thread",
                "register",
                "lead",
                "--role",
                "coordinator",
                "--session",
                "Demo lead",
            ]);
            let made = run.call(target, "", &lead);
            keep(run, target, made);
        }
    }
}

fn keep(run: &mut Runner, project: &str, result: Result<String, String>) {
    match result {
        Ok(_) => run.passed += 1,
        Err(why) => run.failed.push(format!("{project} board: {why}")),
    }
}

fn project_of<'a>(run: &'a Runner, project: &'a str) -> &'a str {
    if project.is_empty() {
        run.key()
    } else {
        project
    }
}

fn request(run: &Runner, spec: &DepSpec, item: &str) -> AskRequest {
    let from = project_of(run, spec.from);
    AskRequest {
        project: from.to_owned(),
        item: item.to_owned(),
        actor: format!("{from} lead"),
        target: project_of(run, spec.target).to_owned(),
        to: None,
        title: spec.title.to_owned(),
        description: Some("Made by coord demo seed.".to_owned()),
        ac: vec![spec.ac.to_owned()],
        onto: None,
    }
}

fn one_request(run: &mut Runner, spec: &DepSpec) -> Result<(), String> {
    let item = run
        .bound(spec.item)
        .ok_or("the requester item was never made")?
        .to_owned();
    let req = request(run, spec, &item);
    let roster = BoardRoster::new(run.home());
    let made = ask(run.home(), &req, &roster).map_err(|e| e.to_string())?;
    if spec.done {
        let closing = CloseRequest {
            how: "done".to_owned(),
            reason: None,
            actor: format!("{} lead", req.target),
        };
        close(run.home(), &made.dep.id, &closing).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// The requests between boards: open and done ones from the demo board, one coming in.
pub(super) fn dependencies(run: &mut Runner) {
    let key = run.key().to_owned();
    let init = words(&["portfolio", "init"]);
    let made = run.call(&key, "", &init);
    keep(run, "portfolio", made);
    for spec in OUTGOING.iter().chain([&INCOMING]) {
        match one_request(run, spec) {
            Ok(()) => run.passed += 1,
            Err(why) => run.failed.push(format!("request {}: {why}", spec.title)),
        }
    }
}

/// Records the merge of one pull request, through the board op: the real verb runs Brock.
pub(super) fn merge(run: &mut Runner) {
    let Some(id) = run.bound(MERGED_PR).map(str::to_owned) else {
        run.failed
            .push("merge: the pull request was never opened".to_owned());
        return;
    };
    let op = json!({
        "kind": "pr-merge", "target": { "coll": "prs", "id": id },
        "set": { "mergeCommit": MERGE_COMMIT },
    });
    let who = Who {
        thread: Some(argus_board::demo::LEAD.to_owned()),
        role: Role::Coordinator,
        actor: None,
    };
    let key = run.key().to_owned();
    let done = with_board(run.home(), &key, false, |board| {
        Ok(apply_op(board, &op, &who, None).outcome)
    });
    match done {
        Ok(text) if text.ends_with("merged") => run.passed += 1,
        Ok(text) => run.failed.push(format!("merge: {text}")),
        Err(e) => run.failed.push(format!("merge: {e}")),
    }
}
