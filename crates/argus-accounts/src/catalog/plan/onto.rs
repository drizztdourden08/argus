//! The one-way plan: the sidebar state of the current (source) scope is copied onto the next
//! (target) scope, so the target ends up as the source is.
//!
//! - A chat the target lacks is copied (unless the target deleted it: its tombstone stands).
//! - A chat both hold takes the source's record whole: title, history, archived state, lineage
//!   and artifacts. Only keys the source lacks stay as the target has them.
//! - A chat only the target holds is KEPT. Nothing is ever deleted, so a one-way copy cannot
//!   lose data; it only overwrites what the two scopes share.
//! - The source is never written, and the target's archive index follows its records.
//!
//! The newest activity does not decide here: the source wins, whichever side was used last.
//! A second plan made right after applying the first is empty.

use std::collections::HashMap;

use serde_json::Value;

use super::{ChangeKind, Loaded, Overlay, Plan, Planner, apply_overlay, load_scope};
use crate::catalog::merge::{fill_missing, full_record};
use crate::config::ArchiveRule;
use crate::ports::{Catalog, Transcripts};
use crate::types::Scope;

const SOURCE: usize = 0;
const TARGET: usize = 1;

/// Plans the copy of the `source` scope onto the `target` scope. A scope that has no chat
/// folder yet gives an empty plan: there is nothing to copy from or onto.
///
/// # Errors
/// When both scopes are the same, or a record, an index or a file cannot be read.
pub fn build_plan_onto(
    catalog: &dyn Catalog,
    transcripts: &dyn Transcripts,
    source: &Scope,
    target: &Scope,
) -> Result<Plan, String> {
    if source == target {
        return Err("a copy needs two different accounts".to_owned());
    }
    let present = catalog.scopes()?;
    if !present.contains(source) || !present.contains(target) {
        return Ok(Plan::default());
    }
    let scopes = vec![
        load_scope(catalog, source.clone())?,
        load_scope(catalog, target.clone())?,
    ];
    let mut planner = Planner {
        transcripts,
        rule: ArchiveRule::default(),
        scopes,
        plan: Plan::default(),
        valid: HashMap::new(),
    };
    planner.run_onto();
    Ok(planner.finish())
}

impl Planner<'_> {
    fn run_onto(&mut self) {
        let ids: Vec<String> = self.scopes[SOURCE].records.keys().cloned().collect();
        for id in &ids {
            self.carry(id);
        }
        self.plan_index(TARGET);
    }

    /// One chat of the source: copied onto the target, or made to match the target's copy.
    fn carry(&mut self, id: &str) {
        let source = self.scopes[SOURCE].records[id].value.clone();
        if !self.has_transcript(&source) {
            self.plan.missing_transcripts.push(id.to_owned());
            return;
        }
        let overlay = self.overlay(&[SOURCE], std::slice::from_ref(&source), id);
        let copies = [source];
        if self.scopes[TARGET].records.contains_key(id) {
            self.match_source(id, &copies, &overlay);
        } else {
            self.copy(TARGET, id, &copies, &overlay);
        }
    }

    fn match_source(&mut self, id: &str, copies: &[Value], overlay: &Overlay) {
        let current = self.scopes[TARGET].records[id].value.clone();
        let mut record = full_record(copies);
        fill_missing(&mut record, std::slice::from_ref(&current));
        apply_overlay(&mut record, overlay);
        if record == current {
            return;
        }
        let loaded: &mut Loaded = self.scopes[TARGET]
            .records
            .get_mut(id)
            .expect("target holds the record");
        let before = loaded.sha.clone();
        loaded.value = record.clone();
        self.push(TARGET, ChangeKind::UpdateSession, id, before, record);
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::build_plan_onto;
    use crate::catalog::apply;
    use crate::catalog::fixtures::{Fixture, record, scope, session_id};
    use crate::catalog::plan::{ChangeKind, Plan};
    use crate::types::Scope;

    fn onto(fx: &Fixture, source: &Scope, target: &Scope) -> Plan {
        build_plan_onto(&fx.catalog(), &fx.transcripts(), source, target).unwrap()
    }

    fn setup() -> (Fixture, Scope, Scope) {
        let fx = Fixture::new();
        let (a, b) = (scope("a1", "o1"), scope("a2", "o2"));
        fx.ensure(&a);
        fx.ensure(&b);
        (fx, a, b)
    }

    #[test]
    fn a_shared_chat_takes_the_values_of_the_source() {
        let (fx, a, b) = setup();
        let mut mine = record(1, 10);
        mine["title"] = json!("from the source");
        mine["isArchived"] = json!(true);
        let mut theirs = record(1, 99);
        theirs["title"] = json!("stale on the target");
        theirs["customKey"] = json!("kept");
        fx.put(&a, 1, &mine);
        fx.put(&b, 1, &theirs);
        fx.transcript(1);
        let plan = onto(&fx, &a, &b);
        let change = plan
            .changes
            .iter()
            .find(|c| c.kind == ChangeKind::UpdateSession)
            .unwrap();
        assert_eq!(change.scope, b);
        assert_eq!(change.data["title"], "from the source");
        assert_eq!(change.data["lastActivityAt"], 10);
        assert_eq!(change.data["isArchived"], true);
        assert_eq!(change.data["customKey"], "kept");
        assert_eq!(plan.conflicts.len(), 0);
    }

    #[test]
    fn a_chat_only_the_source_has_is_added() {
        let (fx, a, b) = setup();
        fx.put(&a, 1, &record(1, 10));
        fx.transcript(1);
        let plan = onto(&fx, &a, &b);
        let copies: Vec<_> = plan
            .changes
            .iter()
            .filter(|c| c.kind == ChangeKind::CopySession)
            .collect();
        assert_eq!(copies.len(), 1);
        assert_eq!(copies[0].scope, b);
        assert_eq!(copies[0].session_id, session_id(1));
    }

    #[test]
    fn a_chat_only_the_target_has_is_kept_and_the_source_is_never_written() {
        let (fx, a, b) = setup();
        fx.put(&a, 1, &record(1, 10));
        fx.put(&b, 2, &record(2, 50));
        fx.transcript(1);
        fx.transcript(2);
        let plan = onto(&fx, &a, &b);
        assert!(plan.changes.iter().all(|c| c.scope == b));
        assert!(plan.changes.iter().all(|c| c.session_id != session_id(2)));
        let deletes = plan
            .changes
            .iter()
            .filter(|c| c.kind == ChangeKind::DeleteSession)
            .count();
        assert_eq!(deletes, 0);
    }

    #[test]
    fn a_second_plan_after_applying_is_empty() {
        let (fx, a, b) = setup();
        let mut shared = record(1, 10);
        shared["isArchived"] = json!(true);
        fx.put(&a, 1, &shared);
        fx.put(&b, 1, &record(1, 99));
        fx.put(&a, 3, &record(3, 5));
        fx.put(&b, 2, &record(2, 50));
        for n in 1..=3 {
            fx.transcript(n);
        }
        let plan = onto(&fx, &a, &b);
        assert!(!plan.changes.is_empty());
        apply(&fx.catalog(), &plan, |_, _| {}).unwrap();
        let again = onto(&fx, &a, &b);
        assert_eq!(again.changes.len(), 0, "{:?}", again.changes);
    }

    #[test]
    fn a_chat_the_target_deleted_is_not_brought_back() {
        let (fx, a, b) = setup();
        fx.put(&a, 1, &record(1, 10));
        fx.transcript(1);
        fx.tombstone(&b, 1);
        let plan = onto(&fx, &a, &b);
        assert_eq!(plan.changes.len(), 0);
        assert_eq!(plan.tombstone_skips.len(), 1);
    }
}
