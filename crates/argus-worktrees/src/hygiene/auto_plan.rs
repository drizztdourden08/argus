//! Automatic repo rules in report mode: the hygiene leaves their worktrees alone, and the plan lists what it would do.

use super::judge::Verdict;
use crate::auto::AutoMode;
use crate::board::BoardFacts;
use crate::class::Class;
use crate::fact::Fact;
use crate::service::Service;
use serde::Serialize;

/// What the hygiene would do with one worktree under an automatic rule.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanRow {
    /// The board whose lead would be asked.
    pub board: String,
    pub path: String,
    pub class: Class,
    pub would_remove: bool,
    pub reason: String,
}

fn reason_of(verdict: &Verdict) -> String {
    match verdict {
        Verdict::Quiet => {
            "not the hygiene's (a main checkout or a folder git does not list)".to_string()
        }
        Verdict::Attached(why) => format!("attached: {why}"),
        Verdict::Exception(why) => format!("kept as an exception: {why}"),
        Verdict::Clean => "nothing is attached and nothing would be lost".to_string(),
        Verdict::Left(what) => format!("would ask its lead: {what}"),
    }
}

fn row_of(project: &str, fact: &Fact, verdict: &Verdict) -> PlanRow {
    PlanRow {
        board: project.to_string(),
        path: fact.path.clone(),
        class: Class::of(fact),
        would_remove: *verdict == Verdict::Clean,
        reason: reason_of(verdict),
    }
}

impl Service {
    /// Does the hygiene leave this worktree alone: its rule is automatic and the mode is report?
    pub(crate) fn reports_only(&self, board: &BoardFacts, rule_id: &str) -> bool {
        self.opts.auto_rules == AutoMode::Report
            && board.rules.iter().any(|r| r.auto && r.id == rule_id)
    }

    /// What the hygiene does under automatic rules.
    pub fn auto_mode(&self) -> AutoMode {
        self.opts.auto_rules
    }

    /// The would-be verdicts of every worktree under an automatic rule, from the last scans. Read only.
    pub fn auto_plan(&self) -> Vec<PlanRow> {
        let mut rows = Vec::new();
        for project in self.deps.store.projects() {
            let Some(board) = self.deps.store.board(&project) else {
                continue;
            };
            let Some(scanned_ms) = self.clean_scan_ms(&project) else {
                continue;
            };
            rows.extend(
                self.verdicts(&project, &board, scanned_ms)
                    .iter()
                    .filter(|(fact, ..)| board.rules.iter().any(|r| r.auto && r.id == fact.rule))
                    .map(|(fact, verdict, _)| row_of(&project, fact, verdict)),
            );
        }
        rows.sort_by(|a, b| (&a.board, &a.path).cmp(&(&b.board, &b.path)));
        rows
    }
}
