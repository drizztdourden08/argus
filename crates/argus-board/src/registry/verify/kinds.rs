//! Checks that hold for every kind, whichever list it is in: names, codes, looks, tabs, links.

use std::collections::{HashMap, HashSet};

use super::super::kinds::{Family, Kind, Look};
use super::super::model::Registry;
use super::vocab::{
    ASSIGNEE_WORDS, CATEGORY_COLORS, ICONS, ITEM_LETTER, TABS, TONES, TYPE_COLLECTIONS,
};

/// The name of the list a kind comes from, as the problems print it.
fn list_of(kind: &Kind<'_>) -> &'static str {
    match kind.family {
        Family::Type => "types",
        Family::SubItem => "subItems",
        Family::Record => "records",
    }
}

/// Every problem in the kinds taken together and each by itself.
pub(super) fn verify(reg: &Registry, out: &mut Vec<String>) {
    let kinds = reg.kinds();
    verify_names(&kinds, out);
    verify_codes(&kinds, out);
    verify_colors(&kinds, out);
    for kind in &kinds {
        let mut say = |text: &str| out.push(format!("{}.{}: {text}", list_of(kind), kind.id));
        verify_look(kind, &mut say);
        verify_tabs(kind, &mut say);
        verify_collection(kind, &mut say);
        verify_links(&kinds, kind, &mut say);
    }
    verify_records(reg, out);
}

fn verify_names(kinds: &[Kind<'_>], out: &mut Vec<String>) {
    let mut seen = HashSet::new();
    for kind in kinds.iter().filter(|k| !seen.insert(k.id)) {
        out.push(format!("kinds: {} is listed twice", kind.id));
    }
}

fn is_letters(text: &str) -> bool {
    (1..=2).contains(&text.len()) && text.chars().all(|c| c.is_ascii_uppercase())
}

/// Every letter group a kind answers to, with the kind that owns it.
fn spellings<'a>(kinds: &[Kind<'a>]) -> Vec<(&'a str, Kind<'a>)> {
    let mut all = Vec::new();
    for kind in kinds.iter().filter(|k| k.shares.is_none()) {
        all.push((kind.code, *kind));
        all.extend(kind.aliases.iter().map(|a| (a.as_str(), *kind)));
    }
    all
}

fn verify_codes(kinds: &[Kind<'_>], out: &mut Vec<String>) {
    let mut owners: HashMap<&str, &str> = HashMap::new();
    for (letters, kind) in spellings(kinds) {
        if let Some(first) = owners.insert(letters, kind.id) {
            out.push(format!("codes: {letters} names {first} and {}", kind.id));
        }
    }
    for kind in kinds {
        let mut say = |text: &str| out.push(format!("{}.{}: {text}", list_of(kind), kind.id));
        verify_letters(kind, &mut say);
        verify_shares(kinds, kind, &mut say);
    }
}

fn verify_letters(kind: &Kind<'_>, say: &mut impl FnMut(&str)) {
    let spelled = std::iter::once(kind.code).chain(kind.aliases.iter().map(String::as_str));
    if let Some(bad) = spelled.chain(kind.stored).find(|c| !is_letters(c)) {
        say(&format!("code {bad} is one or two capital letters"));
    }
    if kind.answers_to(ITEM_LETTER) {
        say(&format!(
            "the letter {ITEM_LETTER} is the stored form of an item"
        ));
    }
}

fn verify_shares(kinds: &[Kind<'_>], kind: &Kind<'_>, say: &mut impl FnMut(&str)) {
    let Some(target) = kind.shares else { return };
    match kinds.iter().find(|k| k.id == target) {
        None => say(&format!(
            "shares a code with {target}, which does not exist"
        )),
        Some(t) if t.code != kind.code || t.shares.is_some() => {
            say(&format!(
                "its code must be the code of {target}, and {target} must not share one"
            ));
        }
        Some(_) => {}
    }
}

fn verify_colors(kinds: &[Kind<'_>], out: &mut Vec<String>) {
    let mut owners: HashMap<&str, &str> = HashMap::new();
    for kind in kinds {
        if let Look::Color(color) = kind.look {
            if let Some(first) = owners.insert(color, kind.id) {
                out.push(format!(
                    "colours: {color} is worn by {first} and {}",
                    kind.id
                ));
            }
        }
    }
}

fn verify_look(kind: &Kind<'_>, say: &mut impl FnMut(&str)) {
    if !ICONS.contains(&kind.icon) {
        say(&format!("unknown icon {}", kind.icon));
    }
    match kind.look {
        Look::Color(c) if !CATEGORY_COLORS.contains(&c) => say(&format!("unknown colour {c}")),
        Look::Tone(t) if !TONES.contains(&t) => say(&format!("unknown tone {t}")),
        _ => {}
    }
}

fn verify_tabs(kind: &Kind<'_>, say: &mut impl FnMut(&str)) {
    if kind.tabs.is_empty() {
        say("a kind has tabs");
    }
    if let Some(tab) = kind.tabs.iter().find(|t| !TABS.contains(&t.as_str())) {
        say(&format!("unknown tab {tab}"));
    }
    let unique: HashSet<_> = kind.tabs.iter().collect();
    if unique.len() != kind.tabs.len() {
        say("a tab is listed twice");
    }
    let leads_with_overview = kind.tabs.first().map(String::as_str) == Some("overview");
    if kind.family != Family::Record && !leads_with_overview {
        say("the first tab must be overview");
    }
}

fn verify_collection(kind: &Kind<'_>, say: &mut impl FnMut(&str)) {
    let of_a_type = TYPE_COLLECTIONS.contains(&kind.collection);
    if kind.family == Family::Type && !of_a_type {
        say("a type lives in items, tests, launchers or worktrees");
    }
    if kind.family == Family::Record && of_a_type {
        say("a record that is not an item type lives in a collection of its own");
    }
}

fn verify_links(kinds: &[Kind<'_>], kind: &Kind<'_>, say: &mut impl FnMut(&str)) {
    let mut fields = HashSet::new();
    for link in kind.links {
        if !fields.insert(link.field.as_str()) {
            say(&format!("link {} is listed twice", link.field));
        }
        if link.to.is_empty() {
            say(&format!("link {} points to no kind", link.field));
        }
        if let Some(name) = link
            .to
            .iter()
            .find(|n| !kinds.iter().any(|k| k.id == n.as_str()))
        {
            say(&format!(
                "link {} points to unknown kind {name}",
                link.field
            ));
        }
    }
    if let Some(by) = kind.group_by.filter(|g| !fields.contains(g)) {
        say(&format!("groupBy {by} is not one of its links"));
    }
    if kind.assignee.starts_with('=') && !ASSIGNEE_WORDS.contains(&kind.assignee) {
        say(&format!("unknown assignee word {}", kind.assignee));
    }
}

/// What only a record can get wrong: a colour and a tone together, or neither.
fn verify_records(reg: &Registry, out: &mut Vec<String>) {
    for r in &reg.records {
        if r.color.is_some() == r.tone.is_some() {
            out.push(format!(
                "records.{}: set exactly one of color and tone",
                r.id
            ));
        }
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::testing::refused;

    #[test]
    fn two_kinds_with_the_same_code_are_found_but_a_shared_code_is_allowed() {
        let text = refused(|d| d["types"][1]["code"] = json!("E"));
        assert!(text.contains("codes: E names epic and story"), "{text}");
        let text = refused(|d| d["records"][8]["aliases"] = json!(["B"]));
        assert!(text.contains("codes: B names bug and message"), "{text}");
        let text = refused(|d| d["records"][6]["code"] = json!("Z"));
        assert!(text.contains("its code must be the code of note"), "{text}");
    }

    #[test]
    fn a_code_that_is_not_one_or_two_capitals_or_is_the_item_letter_is_found() {
        let text = refused(|d| d["types"][2]["code"] = json!("kk"));
        assert!(
            text.contains("code kk is one or two capital letters"),
            "{text}"
        );
        let text = refused(|d| d["types"][2]["code"] = json!("I"));
        assert!(text.contains("the letter I is the stored form"), "{text}");
        let text = refused(|d| d["types"][2]["code"] = json!("KKK"));
        assert!(text.contains("code KKK is one or two"), "{text}");
    }

    #[test]
    fn a_category_colour_worn_twice_or_a_record_with_both_looks_is_found() {
        let text = refused(|d| d["types"][7]["color"] = json!("violet"));
        assert!(text.contains("violet is worn by epic and build"), "{text}");
        let text = refused(|d| d["records"][0]["tone"] = json!("info"));
        assert!(text.contains("set exactly one of color and tone"), "{text}");
        let text = refused(|d| d["records"][1]["tone"] = json!("loud"));
        assert!(text.contains("unknown tone loud"), "{text}");
    }

    #[test]
    fn a_tab_a_link_or_a_group_that_is_not_defined_is_found() {
        let text = refused(|d| d["records"][2]["tabs"] = json!(["overview", "chat"]));
        assert!(text.contains("records.release: unknown tab chat"), "{text}");
        let text = refused(|d| d["subItems"][0]["tabs"] = json!(["discussion"]));
        assert!(text.contains("the first tab must be overview"), "{text}");
        let text = refused(|d| d["types"][3]["links"][0]["to"] = json!(["moon"]));
        assert!(text.contains("points to unknown kind moon"), "{text}");
        let text = refused(|d| d["types"][3]["groupBy"] = json!("nothing"));
        assert!(
            text.contains("groupBy nothing is not one of its links"),
            "{text}"
        );
        let text = refused(|d| d["types"][3]["assignee"] = json!("=nobody"));
        assert!(text.contains("unknown assignee word =nobody"), "{text}");
    }

    #[test]
    fn a_kind_listed_twice_across_the_lists_is_found() {
        let text = refused(|d| d["records"][0]["id"] = json!("bug"));
        assert!(text.contains("kinds: bug is listed twice"), "{text}");
    }

    #[test]
    fn a_record_in_an_item_collection_and_a_type_outside_them_are_found() {
        let text = refused(|d| d["records"][0]["collection"] = json!("items"));
        assert!(text.contains("a collection of its own"), "{text}");
        let text = refused(|d| d["types"][0]["collection"] = json!("agents"));
        assert!(text.contains("a type lives in items"), "{text}");
    }
}
