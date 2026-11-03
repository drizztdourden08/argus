//! Test support and tests: runs the shared case file against the checker, and pins the rich-v1
//! cases of the design by name.

use serde_json::{Value, json};

use super::{Context, check, check_keyed};
use crate::registry::{ContentType, registry};

/// The case file the web runs too.
const CASES: &str = include_str!("../../registry/content-cases.json");

fn cases() -> Vec<Value> {
    let doc: Value = serde_json::from_str(CASES).unwrap();
    doc["cases"].as_array().unwrap().clone()
}

fn numbers(case: &Value) -> Vec<u32> {
    let list = case["context"]["images"].as_array();
    list.into_iter()
        .flatten()
        .map(|n| u32::try_from(n.as_u64().unwrap()).unwrap())
        .collect()
}

fn texts(case: &Value, key: &str) -> Option<Vec<String>> {
    let list = case["context"][key].as_array()?;
    Some(
        list.iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect(),
    )
}

/// The verdict of one case: `None` when it holds, else what differed.
fn verdict(case: &Value) -> Option<String> {
    let images = numbers(case);
    let ids = texts(case, "ids");
    let threads = texts(case, "threads");
    let cx = Context {
        images: &images,
        ids: ids.as_deref(),
        threads: threads.as_deref(),
    };
    let got = check_keyed(case["field"].as_str().unwrap(), &case["value"], &cx);
    let name = &case["name"];
    match (case["ok"].as_bool().unwrap(), got) {
        (true, Ok(value)) => case
            .get("result")
            .filter(|want| **want != value)
            .map(|want| format!("{name}: got {value}, want {want}")),
        (true, Err(r)) => Some(format!("{name}: refused with {}: {}", r.key, r.message)),
        (false, Ok(value)) => Some(format!("{name}: accepted as {value}")),
        (false, Err(r)) => {
            (case["key"] != r.key).then(|| format!("{name}: key {}, want {}", r.key, case["key"]))
        }
    }
}

fn text_of(count: usize, line: &str) -> String {
    vec![line; count].join("\n")
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_case_of_the_shared_file_gets_its_verdict() {
        let all = cases();
        assert!(all.len() > 100, "the case file lost cases");
        let failed: Vec<String> = all.iter().filter_map(verdict).collect();
        assert!(failed.is_empty(), "{}", failed.join("\n"));
    }

    #[test]
    fn the_case_file_covers_every_content_kind_and_names_only_known_fields() {
        let used: Vec<ContentType> = cases()
            .iter()
            .filter_map(|c| registry().field(c["field"].as_str().unwrap()))
            .map(|f| f.content)
            .collect();
        for kind in [
            ContentType::Line,
            ContentType::Slug,
            ContentType::Rich,
            ContentType::Enum,
            ContentType::Ref,
            ContentType::Refs,
            ContentType::Date,
            ContentType::Person,
            ContentType::Records,
            ContentType::Flag,
        ] {
            assert!(used.contains(&kind), "no case for {kind:?}");
        }
    }

    #[test]
    fn rich_a_lone_paragraph_is_refused_with_the_example() {
        let cx = Context::default();
        let long = json!(format!("{} end", "word ".repeat(30)));
        let err = check("description", &long, &cx).unwrap_err().to_string();
        assert!(err.contains("a lone paragraph is not accepted"), "{err}");
        assert!(err.contains("Example"), "{err}");
    }

    #[test]
    fn rich_bullets_and_a_lead_line_are_accepted() {
        let cx = Context::default();
        let text =
            json!("Make the lobby clearer\n- seats show who joined\n- approve button is visible");
        assert!(check("description", &text, &cx).is_ok());
    }

    #[test]
    fn rich_an_image_the_record_does_not_have_is_refused() {
        let have = [1_u32];
        let cx = Context {
            images: &have,
            ..Context::default()
        };
        let text = json!("- see\n![lobby](img:2)");
        let err = check_keyed("description", &text, &cx).unwrap_err();
        assert_eq!(err.key, "rich-image-unknown");
        assert!(check("description", &json!("- see\n![lobby](img:1)"), &cx).is_ok());
    }

    #[test]
    fn rich_500_characters_of_text_pass_and_501_are_refused() {
        let cx = Context::default();
        let bullet = format!("- {}", "x".repeat(50));
        let ten = text_of(10, &bullet);
        assert!(check("description", &json!(ten), &cx).is_ok());
        let over = format!("{ten}\n- x");
        let err = check_keyed("description", &json!(over), &cx).unwrap_err();
        assert_eq!(err.key, "rich-text-long");
        assert!(err.message.contains("501"), "{}", err.message);
    }

    #[test]
    fn a_refusal_through_check_is_a_board_refusal_with_the_message() {
        let err = check("title", &json!("a".repeat(81)), &Context::default()).unwrap_err();
        assert_eq!(
            err.to_string(),
            "title is 81 chars, the limit is 80. Shorten it."
        );
    }
}
