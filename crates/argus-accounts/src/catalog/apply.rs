//! Applies a plan: checks that no file changed since the plan was made, then writes each change
//! atomically, reporting every step. No backup is made here; the sync job takes one before it
//! calls [`apply`].

use std::collections::HashSet;

use serde_json::Value;

use super::plan::{Change, ChangeKind, Plan};
use crate::ports::Catalog;

/// Where one change is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStatus {
    /// The write is about to start.
    Running,
    /// The file is written.
    Done,
    /// The write failed; the text says why. The run stops here.
    Failed(String),
}

/// Checks that every file of the plan is exactly as it was when the plan was made, and that no
/// file is in the plan twice.
///
/// # Errors
/// With the first file that changed, is missing from the plan's expectation, or repeats.
pub fn preflight(catalog: &dyn Catalog, plan: &Plan) -> Result<(), String> {
    let mut seen = HashSet::new();
    for change in &plan.changes {
        if !seen.insert((change.scope.name(), change.file.clone())) {
            return Err(format!("the plan writes {} twice", label(change)));
        }
        let now = catalog.file_sha256(&change.scope, &change.file)?;
        if now != change.before_sha256 {
            return Err(format!(
                "{} changed after the plan was made; plan again",
                label(change)
            ));
        }
    }
    Ok(())
}

fn label(change: &Change) -> String {
    format!("{}/{}", change.scope.name(), change.file)
}

fn write(catalog: &dyn Catalog, change: &Change) -> Result<(), String> {
    match change.kind {
        ChangeKind::CopySession | ChangeKind::UpdateSession => {
            catalog.write_session(&change.scope, &change.session_id, &change.data)
        }
        ChangeKind::ArchiveIndex => {
            let ids = change
                .data
                .get("archived")
                .and_then(Value::as_array)
                .and_then(|a| {
                    a.iter()
                        .map(|s| s.as_str().map(str::to_owned))
                        .collect::<Option<Vec<_>>>()
                })
                .ok_or_else(|| format!("{} has no archive list", label(change)))?;
            catalog.write_archived(&change.scope, &ids)
        }
    }
}

/// Writes the plan, in order. Each change is reported as `Running`, then `Done` or `Failed`; a
/// failure stops the run, leaving the files before it written (the backup undoes them).
///
/// # Errors
/// When the preflight fails (nothing is written) or a write fails.
pub fn apply(
    catalog: &dyn Catalog,
    plan: &Plan,
    mut on_step: impl FnMut(&Change, StepStatus),
) -> Result<usize, String> {
    preflight(catalog, plan)?;
    for (done, change) in plan.changes.iter().enumerate() {
        on_step(change, StepStatus::Running);
        if let Err(e) = write(catalog, change) {
            on_step(change, StepStatus::Failed(e.clone()));
            return Err(format!("{}: {e} ({done} written before it)", label(change)));
        }
        on_step(change, StepStatus::Done);
    }
    Ok(plan.changes.len())
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{StepStatus, apply, preflight};
    use crate::catalog::fixtures::{Fixture, record, scope, session_id, three_scopes};
    use crate::catalog::plan::{ChangeKind, build_plan};
    use crate::config::ArchiveRule;
    use crate::ports::Catalog;

    fn busy_fixture() -> Fixture {
        let fx = Fixture::new();
        let (a, b, c) = three_scopes();
        fx.ensure(&c);
        let mut old = record(1, 10);
        old["permissionMode"] = json!("plan");
        old["isArchived"] = json!(true);
        let mut new = record(1, 20);
        new["title"] = json!("newer");
        new["spawnedFrom"] = json!({ "sessionId": "local_p" });
        fx.put(&a, 1, &old);
        fx.put(&b, 1, &new);
        fx.put(&a, 2, &record(2, 5));
        fx.write_index(&b, &[session_id(7)]);
        fx.transcript(1);
        fx.transcript(2);
        fx
    }

    fn plan_of(fx: &Fixture, rule: ArchiveRule) -> crate::catalog::plan::Plan {
        build_plan(&fx.catalog(), &fx.transcripts(), rule).unwrap()
    }

    #[test]
    fn a_second_plan_after_applying_is_empty() {
        for rule in [ArchiveRule::Newest, ArchiveRule::Anywhere] {
            let fx = busy_fixture();
            let plan = plan_of(&fx, rule);
            assert!(!plan.changes.is_empty());
            let written = apply(&fx.catalog(), &plan, |_, _| {}).unwrap();
            assert_eq!(written, plan.changes.len());
            let again = plan_of(&fx, rule);
            assert!(again.changes.is_empty(), "{:?}", again.changes);
            assert!(again.conflicts.is_empty());
        }
    }

    #[test]
    fn every_change_is_reported_running_then_done() {
        let fx = busy_fixture();
        let plan = plan_of(&fx, ArchiveRule::Newest);
        let mut steps = Vec::new();
        apply(&fx.catalog(), &plan, |c, s| steps.push((c.file.clone(), s))).unwrap();
        assert_eq!(steps.len(), plan.changes.len() * 2);
        assert_eq!(steps[0].1, StepStatus::Running);
        assert_eq!(steps[1].1, StepStatus::Done);
        assert_eq!(steps[0].0, steps[1].0);
    }

    #[test]
    fn a_file_changed_since_the_plan_blocks_everything() {
        let fx = busy_fixture();
        let plan = plan_of(&fx, ArchiveRule::Newest);
        let update = plan
            .changes
            .iter()
            .find(|c| c.kind == ChangeKind::UpdateSession)
            .unwrap();
        let mut touched = record(1, 99);
        touched["title"] = json!("edited meanwhile");
        fx.put(&update.scope, 1, &touched);
        assert!(preflight(&fx.catalog(), &plan).is_err());
        let mut steps = 0;
        assert!(apply(&fx.catalog(), &plan, |_, _| steps += 1).is_err());
        assert_eq!(steps, 0, "nothing is written after a failed preflight");
        let b = scope("a3", "o3");
        assert!(fx.catalog().session_ids(&b).unwrap().is_empty());
    }

    #[test]
    fn a_file_that_appeared_since_the_plan_blocks_a_copy() {
        let fx = busy_fixture();
        let plan = plan_of(&fx, ArchiveRule::Newest);
        let copy = plan
            .changes
            .iter()
            .find(|c| c.kind == ChangeKind::CopySession)
            .unwrap();
        fx.put(&copy.scope, 2, &record(2, 1));
        assert!(preflight(&fx.catalog(), &plan).is_err());
    }
}
