//! Checks on the sub-items: questions, decisions, to-dos and notes. What every kind shares (names,
//! codes, looks, tabs, links) is checked in [`super::kinds`].

use super::super::model::Registry;

/// Every problem in the sub-items.
pub(super) fn verify(reg: &Registry, out: &mut Vec<String>) {
    for s in &reg.sub_items {
        let mut say = |text: &str| out.push(format!("subItems.{}: {text}", s.id));
        if s.code.len() != 1 {
            say("the code of a sub-item is one capital letter");
        }
        if !reg.fields.contains_key(&s.field) {
            say(&format!("unknown field {}", s.field));
        }
    }
}
