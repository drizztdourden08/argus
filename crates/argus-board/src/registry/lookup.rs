//! The questions the item ops ask of the registry: a type, a stage, a field, a parent rule.

use super::model::{Collection, Field, ItemType, Registry, Stage};

impl Registry {
    /// The item type with this id.
    #[must_use]
    pub fn item_type(&self, id: &str) -> Option<&ItemType> {
        self.types.iter().find(|t| t.id == id)
    }

    /// The field with this id.
    #[must_use]
    pub fn field(&self, id: &str) -> Option<&Field> {
        self.fields.get(id)
    }

    /// The length limit of a kind of free text (the table `limits::limit` reads).
    #[must_use]
    pub fn limit(&self, kind: &str) -> Option<usize> {
        self.limits.get(kind).copied()
    }

    /// Whether an item of type `child` may sit under an item of type `parent`.
    #[must_use]
    pub fn allows_parent(&self, child: &str, parent: &str) -> bool {
        self.item_type(child)
            .is_some_and(|t| t.parents.iter().any(|p| p == parent))
    }

    /// Whether a build may serve an item of type `served`.
    #[must_use]
    pub fn can_serve(&self, served: &str) -> bool {
        self.types
            .iter()
            .filter(|t| t.collection == Collection::Launchers)
            .any(|t| t.serves.iter().any(|s| s == served))
    }

    /// Whether `key` is a project whose lead the people list knows.
    #[must_use]
    pub fn knows_lead(&self, key: &str) -> bool {
        self.people.leads.contains_key(key)
    }
}

impl ItemType {
    /// The stage with this id.
    #[must_use]
    pub fn stage(&self, id: &str) -> Option<&Stage> {
        self.stages.iter().find(|s| s.id == id)
    }

    /// The lane a stage sits in.
    #[must_use]
    pub fn lane_of(&self, stage: &str) -> Option<&str> {
        self.stage(stage).map(|s| s.lane.as_str())
    }

    /// The first stage of a lane: where an item lands when only the lane is given.
    #[must_use]
    pub fn first_stage_in(&self, lane: &str) -> Option<&Stage> {
        self.stages.iter().find(|s| s.lane == lane)
    }

    /// Whether the type has this field of its own or as a base field.
    #[must_use]
    pub fn has_field(&self, reg: &Registry, id: &str) -> bool {
        self.fields.iter().any(|f| f == id) || reg.base_fields.iter().any(|f| f == id)
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use crate::registry::registry;

    #[test]
    fn every_type_of_the_design_is_there_in_display_order() {
        let ids: Vec<_> = registry().types.iter().map(|t| t.id.as_str()).collect();
        let want = [
            "epic",
            "story",
            "task",
            "bug",
            "debt",
            "test-plan",
            "test-run",
            "test-case",
            "build",
            "worktree",
            "board",
        ];
        assert_eq!(ids, want);
    }

    #[test]
    fn a_stage_knows_its_lane_and_a_lane_its_first_stage() {
        let bug = registry().item_type("bug").unwrap();
        assert_eq!(bug.lane_of("fixing"), Some("doing"));
        assert_eq!(bug.lane_of("nope"), None);
        assert_eq!(
            bug.first_stage_in("todo").map(|s| s.id.as_str()),
            Some("reported")
        );
        assert_eq!(
            bug.first_stage_in("dropped").map(|s| s.id.as_str()),
            Some("wontfix")
        );
        let story = registry().item_type("story").unwrap();
        assert_eq!(
            story.first_stage_in("todo").map(|s| s.id.as_str()),
            Some("draft")
        );
    }

    #[test]
    fn the_bug_flow_lists_the_moves_of_the_design() {
        let bug = registry().item_type("bug").unwrap();
        let next = |id: &str| bug.stage(id).unwrap().next.clone();
        assert_eq!(
            next("verifying"),
            Some(vec!["closed".to_owned(), "fixing".to_owned()])
        );
        assert_eq!(next("closed"), None);
        let task = registry().item_type("task").unwrap();
        assert!(task.stages.iter().all(|s| s.next.is_none()));
    }

    #[test]
    fn parents_follow_the_table_of_the_design() {
        let reg = registry();
        assert!(reg.allows_parent("story", "epic"));
        assert!(!reg.allows_parent("story", "story"));
        assert!(reg.allows_parent("bug", "story"));
        assert!(!reg.allows_parent("epic", "epic"));
        assert!(!reg.allows_parent("ghost", "epic"));
        assert_eq!(reg.item_type("test-case").unwrap().parents.len(), 0);
    }

    #[test]
    fn done_rules_carry_their_mode() {
        use crate::registry::Mode;
        let done = |id: &str| {
            registry()
                .item_type(id)
                .unwrap()
                .done
                .as_ref()
                .map(|d| d.mode)
        };
        assert_eq!(done("task"), Some(Mode::Warn));
        assert_eq!(done("debt"), Some(Mode::Warn));
        assert_eq!(done("story"), Some(Mode::Refuse));
        assert_eq!(done("bug"), Some(Mode::Refuse));
        assert_eq!(done("test-case"), None);
    }

    #[test]
    fn a_type_has_its_own_and_the_base_fields() {
        let reg = registry();
        let bug = reg.item_type("bug").unwrap();
        assert!(bug.has_field(reg, "severity"));
        assert!(bug.has_field(reg, "title"));
        assert!(!bug.has_field(reg, "goal"));
        assert!(reg.knows_lead("brock"));
        assert!(!reg.knows_lead("nobody"));
    }

    #[test]
    fn the_test_plan_ready_stage_has_its_gates() {
        let plan = registry().item_type("test-plan").unwrap();
        let ready = plan.stage("ready").unwrap();
        let gates: Vec<_> = ready.gates.iter().map(|g| g.need.as_str()).collect();
        assert_eq!(gates, ["cases"]);
        assert_eq!(
            registry()
                .item_type("bug")
                .unwrap()
                .stage("duplicate")
                .unwrap()
                .requires,
            ["duplicateOf"]
        );
    }

    #[test]
    fn a_build_serves_the_work_item_types_and_nothing_else() {
        let reg = registry();
        for served in [
            "epic",
            "story",
            "task",
            "bug",
            "debt",
            "test-plan",
            "test-run",
        ] {
            assert!(reg.can_serve(served), "{served}");
        }
        for other in ["test-case", "build", "worktree", "ghost"] {
            assert!(!reg.can_serve(other), "{other}");
        }
    }

    #[test]
    fn the_build_type_is_keyed_by_name_and_has_the_link_fields() {
        let reg = registry();
        let build = reg.item_type("build").unwrap();
        assert_eq!(build.id_key.as_deref(), Some("name"));
        assert_eq!(build.parents.len(), 0);
        assert_eq!(build.tabs.first().map(String::as_str), Some("overview"));
        assert!(build.has_field(reg, "for"));
        assert!(build.has_field(reg, "master"));
    }

    #[test]
    fn a_build_stage_is_computed_in_three_lanes() {
        let build = registry().item_type("build").unwrap();
        assert_eq!(build.lane_of("draft"), Some("todo"));
        assert_eq!(build.lane_of("ready"), Some("doing"));
        assert_eq!(build.lane_of("closed"), Some("done"));
        assert_eq!(build.lane_of("dropped"), None);
    }

    #[test]
    fn a_worktree_is_a_read_only_record_with_no_stage_and_three_tabs() {
        let wt = registry().item_type("worktree").unwrap();
        assert!(wt.read_only);
        assert_eq!(wt.id_key.as_deref(), Some("id"));
        assert_eq!(wt.stages.len(), 0);
        assert_eq!(wt.tabs, ["overview", "links", "activity"]);
        assert!(wt.has_field(registry(), "linkSource"));
    }

    #[test]
    fn the_work_tab_sits_before_activity_on_the_types_a_build_serves() {
        let reg = registry();
        for id in ["epic", "story", "task", "bug", "debt", "test-plan"] {
            let tabs = &reg.item_type(id).unwrap().tabs;
            let at = tabs.iter().position(|t| t == "work").unwrap();
            assert_eq!(
                tabs.get(at + 1).map(String::as_str),
                Some("activity"),
                "{id}"
            );
        }
        let case = reg.item_type("test-case").unwrap();
        assert!(!case.tabs.iter().any(|t| t == "work"));
        assert_eq!(case.tabs[1], "result");
    }
}
