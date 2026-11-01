//! The result of a comparison and how it is told: per route equal, differs (with the first paths
//! that differ) or missing on one side, a summary line, and the exit code.
// test-tag: critical

use serde_json::{Value, json};

use crate::diff::Change;
use crate::verdict::Verdict;

/// How many characters of a value a text line shows.
const SHOWN: usize = 70;

/// The verdict on one route.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteResult {
    /// The route as `GET /api/health`.
    pub label: String,
    /// How it compares.
    pub verdict: Verdict,
}

/// The whole comparison.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    /// The left server.
    pub left: String,
    /// The right server.
    pub right: String,
    /// One result per route, in the order they were read.
    pub routes: Vec<RouteResult>,
    /// The routes left out by `--skip`.
    pub skipped: Vec<String>,
    /// Findings that are not a route difference: a board file that is all NUL bytes, empty or
    /// not JSON. Any finding fails the run.
    pub findings: Vec<String>,
}

/// The numbers of the summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    /// Routes that are equal.
    pub equal: usize,
    /// Routes that differ.
    pub differs: usize,
    /// Routes missing on one side.
    pub missing: usize,
}

fn shown(value: Option<&Value>) -> String {
    let Some(v) = value else {
        return "(absent)".to_owned();
    };
    let text = v.to_string();
    if text.chars().count() <= SHOWN {
        return text;
    }
    let head: String = text.chars().take(SHOWN).collect();
    format!("{head}...")
}

fn change_json(c: &Change) -> Value {
    let mut out = json!({ "path": c.path });
    if let Some(l) = &c.left {
        out["left"] = l.clone();
    }
    if let Some(r) = &c.right {
        out["right"] = r.clone();
    }
    out
}

fn verdict_name(v: &Verdict) -> &'static str {
    match v {
        Verdict::Equal => "equal",
        Verdict::Differs(_) => "differs",
        Verdict::Missing { .. } => "missing",
    }
}

fn differs_lines(label: &str, changes: &[Change], max: usize) -> Vec<String> {
    let mut lines = vec![format!("  DIFFERS  {label}  ({} paths)", changes.len())];
    for c in changes.iter().take(max) {
        lines.push(format!(
            "      {}: left {} | right {}",
            c.path,
            shown(c.left.as_ref()),
            shown(c.right.as_ref())
        ));
    }
    if changes.len() > max {
        lines.push(format!("      ... and {} more", changes.len() - max));
    }
    lines
}

impl Report {
    /// The counts of the summary.
    #[must_use]
    pub fn counts(&self) -> Counts {
        let count = |name: &str| {
            self.routes
                .iter()
                .filter(|r| verdict_name(&r.verdict) == name)
                .count()
        };
        Counts {
            equal: count("equal"),
            differs: count("differs"),
            missing: count("missing"),
        }
    }

    /// Whether every route is equal.
    #[must_use]
    pub fn is_equal(&self) -> bool {
        let c = self.counts();
        c.differs == 0 && c.missing == 0 && c.equal > 0 && self.findings.is_empty()
    }

    /// 0 when every route is equal, 1 otherwise.
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        u8::from(!self.is_equal())
    }

    fn summary(&self) -> String {
        let c = self.counts();
        format!(
            "summary: {} equal, {} differ, {} missing, {} skipped, {} findings; exit {}",
            c.equal,
            c.differs,
            c.missing,
            self.skipped.len(),
            self.findings.len(),
            self.exit_code()
        )
    }

    /// The report as lines of text, at most `max_diffs` paths under each differing route.
    #[must_use]
    pub fn text(&self, max_diffs: usize) -> String {
        let mut lines = vec![format!("parity: left {} | right {}", self.left, self.right)];
        lines.extend(self.findings.iter().map(|f| format!("  FINDING  {f}")));
        for route in &self.routes {
            match &route.verdict {
                Verdict::Equal => lines.push(format!("  equal    {}", route.label)),
                Verdict::Differs(changes) => {
                    lines.extend(differs_lines(&route.label, changes, max_diffs));
                }
                Verdict::Missing { side, why } => lines.push(format!(
                    "  MISSING  {}  ({}: {why})",
                    route.label,
                    side.word()
                )),
            }
        }
        lines.extend(self.skipped.iter().map(|s| format!("  skipped  {s}")));
        lines.push(self.summary());
        lines.join("\n") + "\n"
    }

    /// The report as JSON, at most `max_diffs` changes under each differing route.
    #[must_use]
    pub fn json(&self, max_diffs: usize) -> Value {
        let routes: Vec<Value> = self
            .routes
            .iter()
            .map(|r| route_json(r, max_diffs))
            .collect();
        let c = self.counts();
        json!({
            "left": self.left,
            "right": self.right,
            "summary": { "equal": c.equal, "differs": c.differs, "missing": c.missing, "skipped": self.skipped.len(), "findings": self.findings.len() },
            "skipped": self.skipped,
            "findings": self.findings,
            "exit": self.exit_code(),
            "routes": routes,
        })
    }
}

fn route_json(route: &RouteResult, max_diffs: usize) -> Value {
    let mut out = json!({ "route": route.label, "verdict": verdict_name(&route.verdict) });
    match &route.verdict {
        Verdict::Equal => {}
        Verdict::Differs(changes) => {
            out["paths"] = json!(changes.len());
            out["changes"] = changes.iter().take(max_diffs).map(change_json).collect();
        }
        Verdict::Missing { side, why } => {
            out["side"] = json!(side.word());
            out["why"] = json!(why);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Report, RouteResult};
    use crate::diff::Change;
    use crate::verdict::{Verdict, Which};

    fn report(routes: Vec<RouteResult>) -> Report {
        Report {
            left: "http://l".into(),
            right: "http://r".into(),
            routes,
            skipped: Vec::new(),
            findings: Vec::new(),
        }
    }

    fn route(label: &str, verdict: Verdict) -> RouteResult {
        RouteResult {
            label: label.into(),
            verdict,
        }
    }

    fn change(path: &str) -> Change {
        Change {
            path: path.into(),
            left: Some(json!(1)),
            right: None,
        }
    }

    #[test]
    fn all_equal_exits_zero() {
        let r = report(vec![route("GET /a", Verdict::Equal)]);
        assert_eq!(r.exit_code(), 0);
        assert!(
            r.text(5)
                .contains("summary: 1 equal, 0 differ, 0 missing, 0 skipped, 0 findings; exit 0")
        );
    }

    #[test]
    fn a_finding_fails_an_otherwise_equal_run_and_is_told_on_its_own_line() {
        let mut r = report(vec![route("GET /a", Verdict::Equal)]);
        r.findings
            .push("board \"tessera\" is damaged: all NUL".into());
        assert_eq!(r.exit_code(), 1);
        assert!(
            r.text(5)
                .contains("  FINDING  board \"tessera\" is damaged: all NUL")
        );
        let j = r.json(5);
        assert_eq!(j["summary"]["findings"], 1);
        assert_eq!(j["findings"][0], "board \"tessera\" is damaged: all NUL");
    }

    #[test]
    fn a_difference_or_a_missing_route_exits_one() {
        let d = report(vec![route("GET /a", Verdict::Differs(vec![change("x")]))]);
        assert_eq!(d.exit_code(), 1);
        let m = report(vec![route(
            "GET /a",
            Verdict::Missing {
                side: Which::Right,
                why: "HTTP 404".into(),
            },
        )]);
        assert_eq!(m.exit_code(), 1);
        assert!(m.text(5).contains("MISSING  GET /a  (right: HTTP 404)"));
    }

    #[test]
    fn the_text_shows_the_first_paths_and_counts_the_rest() {
        let changes = vec![change("a"), change("b"), change("c")];
        let text = report(vec![route("GET /a", Verdict::Differs(changes))]).text(2);
        assert!(text.contains("DIFFERS  GET /a  (3 paths)"));
        assert!(text.contains("a: left 1 | right (absent)"));
        assert!(text.contains("... and 1 more"));
        assert!(!text.contains("      c:"));
    }

    #[test]
    fn the_json_carries_the_verdicts_and_the_summary() {
        let r = report(vec![
            route("GET /a", Verdict::Equal),
            route("GET /b", Verdict::Differs(vec![change("x")])),
        ]);
        let j = r.json(5);
        assert_eq!(j["summary"]["differs"], 1);
        assert_eq!(j["routes"][1]["changes"][0]["path"], "x");
        assert_eq!(j["exit"], 1);
    }

    #[test]
    fn an_empty_report_is_not_a_pass() {
        assert_eq!(report(Vec::new()).exit_code(), 1);
    }
}
