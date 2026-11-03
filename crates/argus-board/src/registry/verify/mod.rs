//! The cross-reference rules of the registry: every name a part of the file uses is defined
//! somewhere else in it. The shape itself is checked while parsing ([`super::model`]).

mod fields;
mod kinds;
mod people;
mod stages;
mod sub_items;
mod types;
mod vocab;

use std::collections::HashSet;

use super::model::Registry;

/// The only structure version this code reads.
const VERSION: u32 = 1;

/// Every cross-reference problem of `reg`, one line each; empty when the file is consistent.
pub(super) fn problems(reg: &Registry) -> Vec<String> {
    let mut out = Vec::new();
    if reg.version != VERSION {
        out.push(format!("version {} is not {VERSION}", reg.version));
    }
    let lanes: HashSet<_> = reg.lanes.iter().collect();
    if lanes.len() != reg.lanes.len() || lanes.is_empty() {
        out.push("lanes: a non-empty list of different names".to_owned());
    }
    fields::verify(reg, &mut out);
    types::verify(reg, &mut out);
    sub_items::verify(reg, &mut out);
    kinds::verify(reg, &mut out);
    people::verify(reg, &mut out);
    out
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use crate::registry::load::{SOURCE, parse};

    /// The error text of the shipped file after `edit` changed it.
    fn refused(edit: impl FnOnce(&mut Value)) -> String {
        let mut doc: Value = serde_json::from_str(SOURCE).unwrap();
        edit(&mut doc);
        parse(&doc.to_string()).unwrap_err().to_string()
    }

    fn item_type<'a>(doc: &'a mut Value, id: &str) -> &'a mut Value {
        let all = doc["types"].as_array_mut().unwrap();
        all.iter_mut().find(|t| t["id"] == id).unwrap()
    }

    #[test]
    fn the_shipped_file_is_consistent() {
        assert!(parse(SOURCE).is_ok());
    }

    #[test]
    fn an_unknown_key_fails_the_parse() {
        let text = refused(|d| d["types"][0]["colour"] = json!("red"));
        assert!(text.contains("does not parse"), "{text}");
    }

    #[test]
    fn a_parent_that_does_not_exist_is_found() {
        let text = refused(|d| item_type(d, "bug")["parents"] = json!(["ghost"]));
        assert!(text.contains("parent type ghost does not exist"), "{text}");
    }

    #[test]
    fn a_loop_of_parents_is_found() {
        let text = refused(|d| item_type(d, "epic")["parents"] = json!(["task"]));
        assert!(text.contains("nest deeper"), "{text}");
    }

    #[test]
    fn a_stage_in_a_lane_that_does_not_exist_is_found() {
        let text = refused(|d| item_type(d, "task")["stages"][0]["lane"] = json!("later"));
        assert!(text.contains("unknown lane later"), "{text}");
    }

    #[test]
    fn a_lane_with_no_stage_is_found() {
        let text = refused(|d| item_type(d, "task")["stages"][4]["lane"] = json!("done"));
        assert!(text.contains("no stage is in lane dropped"), "{text}");
    }

    #[test]
    fn a_move_to_a_stage_that_does_not_exist_is_found() {
        let text = refused(|d| item_type(d, "bug")["stages"][0]["next"] = json!(["lost"]));
        assert!(text.contains("moves to unknown stage lost"), "{text}");
    }

    #[test]
    fn a_field_that_is_not_defined_is_found() {
        let text = refused(|d| item_type(d, "task")["fields"] = json!(["nothing"]));
        assert!(text.contains("unknown field nothing"), "{text}");
    }

    #[test]
    fn a_stage_that_requires_a_foreign_field_is_found() {
        let text = refused(|d| item_type(d, "task")["stages"][4]["requires"] = json!(["severity"]));
        assert!(text.contains("requires severity"), "{text}");
    }

    #[test]
    fn an_unknown_icon_colour_tab_or_done_condition_is_found() {
        let text = refused(|d| item_type(d, "task")["icon"] = json!("rocket"));
        assert!(text.contains("unknown icon rocket"), "{text}");
        let text = refused(|d| item_type(d, "task")["color"] = json!("beige"));
        assert!(text.contains("unknown colour beige"), "{text}");
        let text = refused(|d| item_type(d, "task")["tabs"] = json!(["overview", "chat"]));
        assert!(text.contains("unknown tab chat"), "{text}");
        let text = refused(|d| item_type(d, "task")["done"]["needs"] = json!(["luck"]));
        assert!(text.contains("unknown done condition luck"), "{text}");
    }

    #[test]
    fn a_ref_to_a_kind_that_does_not_exist_is_found() {
        let text = refused(|d| d["fields"]["foundIn"]["to"] = json!(["moon"]));
        assert!(text.contains("unknown ref kind moon"), "{text}");
    }

    #[test]
    fn a_pattern_that_does_not_compile_fails_the_parse() {
        let text = refused(|d| d["content"]["slug"]["pattern"] = json!("(["));
        assert!(text.contains("does not parse"), "{text}");
    }

    #[test]
    fn a_text_field_without_a_max_is_found() {
        let text = refused(|d| {
            d["fields"]["title"].as_object_mut().unwrap().remove("max");
        });
        assert!(text.contains("fields.title"), "{text}");
    }

    #[test]
    fn a_sub_item_with_a_taken_code_is_found() {
        let text = refused(|d| d["subItems"][1]["code"] = json!("Q"));
        assert!(
            text.contains("codes: Q names question and decision"),
            "{text}"
        );
    }

    #[test]
    fn a_lead_with_a_colour_that_does_not_exist_is_found() {
        let text = refused(|d| d["people"]["leads"]["argus"]["color"] = json!("beige"));
        assert!(
            text.contains("people.argus: unknown colour beige"),
            "{text}"
        );
    }

    #[test]
    fn a_build_that_serves_a_type_that_does_not_exist_or_is_not_a_work_item_is_found() {
        let text = refused(|d| item_type(d, "build")["serves"] = json!(["ghost"]));
        assert!(text.contains("serves type ghost does not exist"), "{text}");
        let text = refused(|d| item_type(d, "build")["serves"] = json!(["test-case"]));
        assert!(
            text.contains("serves type test-case is not a work item type"),
            "{text}"
        );
    }

    #[test]
    fn a_build_without_serves_or_a_work_item_with_an_id_key_is_found() {
        let text = refused(|d| item_type(d, "build")["serves"] = json!([]));
        assert!(
            text.contains("serves is set exactly on a build type"),
            "{text}"
        );
        let text = refused(|d| item_type(d, "task")["idKey"] = json!("name"));
        assert!(text.contains("idKey is set exactly"), "{text}");
        let text = refused(|d| {
            item_type(d, "worktree")
                .as_object_mut()
                .unwrap()
                .remove("idKey");
        });
        assert!(text.contains("idKey is set exactly"), "{text}");
    }

    #[test]
    fn every_type_has_tabs_and_starts_with_overview() {
        let text = refused(|d| item_type(d, "build")["tabs"] = json!([]));
        assert!(text.contains("a kind has tabs"), "{text}");
        let text = refused(|d| item_type(d, "worktree")["tabs"] = json!(["links", "overview"]));
        assert!(text.contains("the first tab must be overview"), "{text}");
        let text = refused(|d| item_type(d, "task")["tabs"] = json!(["links", "overview"]));
        assert!(text.contains("the first tab must be overview"), "{text}");
    }

    #[test]
    fn a_worktree_with_stages_or_a_build_with_an_unknown_lane_is_found() {
        let text = refused(|d| {
            item_type(d, "worktree")["stages"] = json!([{"id": "x", "label": "X", "lane": "todo"}]);
        });
        assert!(text.contains("a worktree type has no stages"), "{text}");
        let text = refused(|d| item_type(d, "build")["stages"][0]["lane"] = json!("later"));
        assert!(text.contains("unknown lane later"), "{text}");
    }

    #[test]
    fn a_flag_field_with_a_max_or_values_is_found() {
        let text = refused(|d| d["fields"]["master"]["max"] = json!(5));
        assert!(text.contains("a max is set exactly"), "{text}");
        let text = refused(|d| d["fields"]["master"]["values"] = json!(["yes"]));
        assert!(
            text.contains("values are set exactly on enum fields"),
            "{text}"
        );
    }
}
