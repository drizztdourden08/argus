//! The verdict on one probe: what the two sides gave, put through the rules and compared.

use serde_json::json;

use crate::diff::{Change, diff};
use crate::got::Got;
use crate::normalize::normalise;
use crate::rules::Rules;

/// A side of the comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Which {
    /// The `--left` server.
    Left,
    /// The `--right` server.
    Right,
}

impl Which {
    /// The word of a report.
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Which::Left => "left",
            Which::Right => "right",
        }
    }
}

/// How one route compares.
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    /// The same by value, volatile fields set aside.
    Equal,
    /// Both answered and the answers differ.
    Differs(Vec<Change>),
    /// One side answered and the other did not.
    Missing {
        /// The side that did not answer.
        side: Which,
        /// What it did instead.
        why: String,
    },
}

fn refusals(left: (u16, &str), right: (u16, &str)) -> Verdict {
    let mut changes = Vec::new();
    if left.0 != right.0 {
        changes.push(Change {
            path: "status".to_owned(),
            left: Some(json!(left.0)),
            right: Some(json!(right.0)),
        });
    } else if left.1 != right.1 {
        changes.push(Change {
            path: "error.code".to_owned(),
            left: Some(json!(left.1)),
            right: Some(json!(right.1)),
        });
    }
    if changes.is_empty() {
        Verdict::Equal
    } else {
        Verdict::Differs(changes)
    }
}

/// Compares what the two sides gave under `rules`.
#[must_use]
pub fn judge(rules: Rules, left: &Got, right: &Got) -> Verdict {
    match (left, right) {
        (Got::Doc(l), Got::Doc(r)) => {
            let changes = diff(
                &normalise(l, rules.spec),
                &normalise(r, rules.spec),
                rules.tolerance_ms,
            );
            if changes.is_empty() {
                Verdict::Equal
            } else {
                Verdict::Differs(changes)
            }
        }
        (Got::Doc(_), other) => Verdict::Missing {
            side: Which::Right,
            why: other.describe(),
        },
        (other, Got::Doc(_)) => Verdict::Missing {
            side: Which::Left,
            why: other.describe(),
        },
        (Got::Refused { status: a, code: x }, Got::Refused { status: b, code: y }) => {
            refusals((*a, x), (*b, y))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Verdict, Which, judge};
    use crate::got::Got;
    use crate::kind::Kind;

    fn doc(v: serde_json::Value) -> Got {
        Got::Doc(v)
    }

    #[test]
    fn equal_docs_are_equal_even_when_only_volatile_fields_differ() {
        let l = doc(json!({ "seq": 3, "docs": [{ "id": "a", "data": { "x": 1 } }] }));
        let r = doc(json!({ "seq": 90, "docs": [{ "id": "a", "data": { "x": 1 } }] }));
        assert_eq!(judge(Kind::Doc("items").rules(0), &l, &r), Verdict::Equal);
    }

    #[test]
    fn unordered_rows_compare_as_sets() {
        let l = doc(json!({ "docs": [{ "id": "a" }, { "id": "b" }] }));
        let r = doc(json!({ "docs": [{ "id": "b" }, { "id": "a" }] }));
        assert_eq!(judge(Kind::Doc("items").rules(0), &l, &r), Verdict::Equal);
    }

    #[test]
    fn a_changed_doc_differs_with_its_path() {
        let l = doc(json!({ "docs": [{ "id": "a", "data": { "title": "one" } }] }));
        let r = doc(json!({ "docs": [{ "id": "a", "data": { "title": "two" } }] }));
        let Verdict::Differs(c) = judge(Kind::Doc("items").rules(0), &l, &r) else {
            panic!("expected a difference");
        };
        assert_eq!(c[0].path, "docs[a].data.title");
    }

    #[test]
    fn an_answer_against_a_refusal_is_missing_on_the_refusing_side() {
        let gone = Got::from_answer(404, json!({ "error": { "code": "not_found" } }));
        let v = judge(Kind::Health.rules(0), &doc(json!({})), &gone);
        assert_eq!(
            v,
            Verdict::Missing {
                side: Which::Right,
                why: "HTTP 404 not_found".into()
            }
        );
        let v = judge(Kind::Health.rules(0), &gone, &doc(json!({})));
        assert!(matches!(
            v,
            Verdict::Missing {
                side: Which::Left,
                ..
            }
        ));
    }

    #[test]
    fn the_same_refusal_on_both_sides_is_equal_and_another_one_differs() {
        let a = Got::from_answer(
            503,
            json!({ "error": { "code": "unavailable", "message": "x" } }),
        );
        let b = Got::from_answer(
            503,
            json!({ "error": { "code": "unavailable", "message": "y" } }),
        );
        let c = Got::from_answer(400, json!({ "error": { "code": "bad_request" } }));
        assert_eq!(judge(Kind::Procs.rules(0), &a, &b), Verdict::Equal);
        assert!(matches!(
            judge(Kind::Procs.rules(0), &a, &c),
            Verdict::Differs(_)
        ));
    }
}
