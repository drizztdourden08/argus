//! The `checks` list of each local pull request: the `C-n` of the check runs whose `pr` is it,
//! oldest first. Brock writes the runs, so the lists follow the check run index: they are
//! brought up to date after every scan of the folder.

use std::collections::BTreeMap;
use std::sync::Arc;

use argus_records::checks::CheckRun;
use serde_json::json;

use crate::Services;

/// The writer named on the board for these changes.
const BY: &str = "argus";

type Grouped = BTreeMap<(String, String), Vec<u32>>;

/// The runs that name a pull request, as `(project, PR-n)` to their numbers, ascending.
fn grouped(runs: &[CheckRun]) -> Grouped {
    let mut by_pr = Grouped::new();
    for run in runs {
        let pr = run.record["pr"].as_str().unwrap_or_default().trim();
        if !pr.is_empty() {
            let key = (run.project.clone(), pr.to_ascii_uppercase());
            by_pr.entry(key).or_default().push(run.n);
        }
    }
    for numbers in by_pr.values_mut() {
        numbers.sort_unstable();
    }
    by_pr
}

fn list_of(numbers: &[u32]) -> Vec<String> {
    numbers.iter().map(|n| format!("C-{n}")).collect()
}

/// Puts the list of every pull request that has runs in line with the index. A pull request the
/// board does not have, or one that already holds the list, is left alone.
pub(crate) async fn fill(svc: Arc<Services>) {
    let (Some(records), Some(sync)) = (svc.records.clone(), svc.sync.clone()) else {
        return;
    };
    if svc.shadow {
        return;
    }
    for ((project, pr), numbers) in grouped(&records.checks.runs()) {
        let wanted = list_of(&numbers);
        let board = svc.boards().board(&project);
        let held = board.as_ref().map(|b| &b["prs"][pr.as_str()]);
        if held.is_none_or(|p| p.is_null() || p["checks"] == json!(wanted)) {
            continue;
        }
        let op = json!({ "kind": "pr-checks", "target": { "c": "prs", "id": pr }, "set": { "checks": wanted } });
        if let Err(e) = sync.apply_pr(&project, op, BY).await {
            let detail = json!({ "project": project, "pr": pr, "error": e.message });
            svc.log.warn("pull request checks not updated", detail);
        }
    }
}
