//! The comparison itself: find the scopes both servers know, put every probe to both, judge
//! each pair, then read one doc stream from both and judge its snapshot frames.
// test-tag: critical

use std::path::PathBuf;

use argus_core::ids::PORTFOLIO;
use serde_json::Value;

use crate::findings::board_findings;
use crate::got::Got;
use crate::kind::Kind;
use crate::probe::Probe;
use crate::probes;
use crate::report::{Report, RouteResult};
use crate::side::Side;
use crate::stream::{self, Entry};
use crate::verdict::judge;

/// What to compare and how.
#[derive(Debug, Clone)]
pub struct Options {
    /// The left server's URL.
    pub left: String,
    /// The right server's URL.
    pub right: String,
    /// The left bearer.
    pub token_left: Option<String>,
    /// The right bearer.
    pub token_right: Option<String>,
    /// Only these projects (all when empty).
    pub projects: Vec<String>,
    /// How far apart two timestamps may be and still be equal.
    pub tolerance_ms: i64,
    /// How long the doc streams are read; 0 skips them.
    pub stream_secs: u64,
    /// Routes and stream snapshot frames whose label contains one of these words are not compared.
    pub skip: Vec<String>,
    /// The coord homes whose board files are checked for damage (none checks nothing).
    pub coord_homes: Vec<PathBuf>,
}

impl Options {
    /// The defaults for two servers: 2 s tolerance, a 5 s stream, every project.
    #[must_use]
    pub fn new(left: &str, right: &str) -> Options {
        Options {
            left: left.to_owned(),
            right: right.to_owned(),
            token_left: None,
            token_right: None,
            projects: Vec::new(),
            tolerance_ms: 2000,
            stream_secs: 5,
            skip: Vec::new(),
            coord_homes: Vec::new(),
        }
    }
}

fn keys_of(got: &Got) -> Vec<String> {
    let Got::Doc(Value::Array(rows)) = got else {
        return Vec::new();
    };
    rows.iter()
        .filter_map(|r| r["key"].as_str().map(str::to_owned))
        .collect()
}

/// The scopes to compare: the projects of both servers, and the portfolio, or the ones asked for.
fn scopes(asked: &[String], left: &Got, right: &Got) -> Vec<String> {
    if !asked.is_empty() {
        return asked.to_vec();
    }
    let mut keys: Vec<String> = keys_of(left).into_iter().chain(keys_of(right)).collect();
    keys.sort();
    keys.dedup();
    keys.push(PORTFOLIO.to_owned());
    keys
}

async fn project_lists(left: &Side, right: &Side) -> Result<(Got, Got), String> {
    let probe = Probe::get(Kind::Projects, "/api/projects");
    let (l, r) = tokio::join!(left.fetch(&probe.request), right.fetch(&probe.request));
    for (name, side, got) in [("left", left, &l), ("right", right, &r)] {
        if matches!(got, Got::Refused { status: 0, .. }) {
            return Err(format!(
                "cannot reach {name} {}: {}",
                side.url(),
                got.describe()
            ));
        }
    }
    Ok((l, r))
}

async fn route(opts: &Options, sides: (&Side, &Side), probe: &Probe) -> RouteResult {
    let (l, r) = tokio::join!(sides.0.fetch(&probe.request), sides.1.fetch(&probe.request));
    RouteResult {
        label: probe.label(),
        verdict: judge(probe.kind.rules(opts.tolerance_ms), &l, &r),
    }
}

fn skipped(opts: &Options, label: &str) -> bool {
    opts.skip.iter().any(|w| label.contains(w.as_str()))
}

fn merge_stream(opts: &Options, scope: &str, left: &[Entry], right: &[Entry]) -> Vec<RouteResult> {
    let mut labels: Vec<&String> = left.iter().map(|e| &e.0).collect();
    for e in right {
        if !labels.contains(&&e.0) {
            labels.push(&e.0);
        }
    }
    let find = |side: &'_ [Entry], label: &str| {
        side.iter().find(|e| e.0 == label).map_or_else(
            || (Kind::Ready, Got::none("no such frame")),
            |e| (e.1, e.2.clone()),
        )
    };
    labels
        .into_iter()
        .map(|label| {
            let (kind, l) = find(left, label);
            let (other, r) = find(right, label);
            let kind = if kind == Kind::Ready { other } else { kind };
            RouteResult {
                label: format!("STREAM /api/p/{scope}/stream {label}"),
                verdict: judge(kind.rules(opts.tolerance_ms), &l, &r),
            }
        })
        .collect()
}

async fn streams(opts: &Options, sides: (&Side, &Side), scope: &str) -> Vec<RouteResult> {
    let (l, r) = tokio::join!(
        stream::read(sides.0, scope, opts.stream_secs),
        stream::read(sides.1, scope, opts.stream_secs)
    );
    merge_stream(opts, scope, &l, &r)
}

/// Splits the routes into the compared ones and the labels of those left out by `--skip`.
fn drop_skipped(opts: &Options, routes: Vec<RouteResult>) -> (Vec<RouteResult>, Vec<String>) {
    let (out, kept): (Vec<_>, Vec<_>) = routes.into_iter().partition(|r| skipped(opts, &r.label));
    (kept, out.into_iter().map(|r| r.label).collect())
}

/// Compares the two servers.
///
/// # Errors
/// A server cannot be reached at all.
pub async fn compare(opts: &Options) -> Result<Report, String> {
    let left = Side::new(&opts.left, opts.token_left.clone());
    let right = Side::new(&opts.right, opts.token_right.clone());
    let (lp, rp) = project_lists(&left, &right).await?;
    let scopes = scopes(&opts.projects, &lp, &rp);
    let mut probes = probes::machine();
    for scope in &scopes {
        probes.extend(probes::scope(scope));
    }
    let mut report = Report {
        left: left.url().to_owned(),
        right: right.url().to_owned(),
        routes: Vec::new(),
        skipped: Vec::new(),
        findings: board_findings(&opts.coord_homes),
    };
    for probe in probes {
        if skipped(opts, &probe.label()) {
            report.skipped.push(probe.label());
        } else {
            report
                .routes
                .push(route(opts, (&left, &right), &probe).await);
        }
    }
    let first = scopes.first().map_or(PORTFOLIO, String::as_str);
    if opts.stream_secs > 0 && !skipped(opts, "STREAM") {
        let (kept, left_out) = drop_skipped(opts, streams(opts, (&left, &right), first).await);
        report.routes.extend(kept);
        report.skipped.extend(left_out);
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Options, drop_skipped, scopes, skipped};
    use crate::got::Got;
    use crate::report::RouteResult;
    use crate::verdict::Verdict;

    #[test]
    fn the_scopes_are_the_union_of_both_lists_and_the_portfolio() {
        let l = Got::Doc(json!([{ "key": "beta" }, { "key": "alpha" }]));
        let r = Got::Doc(json!([{ "key": "alpha" }, { "key": "gamma" }]));
        assert_eq!(
            scopes(&[], &l, &r),
            ["alpha", "beta", "gamma", "_portfolio"].map(String::from)
        );
    }

    #[test]
    fn asked_projects_replace_the_list() {
        let l = Got::Doc(json!([{ "key": "alpha" }]));
        assert_eq!(scopes(&["beta".into()], &l, &l), ["beta".to_owned()]);
    }

    #[test]
    fn skip_words_match_inside_the_label() {
        let mut o = Options::new("a", "b");
        o.skip.push("listeners".into());
        assert!(skipped(&o, "GET /api/p/x/wakes/listeners"));
        assert!(!skipped(&o, "GET /api/health"));
    }

    #[test]
    fn skip_words_drop_the_matching_stream_snapshot_frames() {
        let row = |frame: &str| RouteResult {
            label: format!("STREAM /api/p/x/stream snapshot {frame}"),
            verdict: Verdict::Equal,
        };
        let mut o = Options::new("a", "b");
        o.skip = vec!["wakes".into(), "snapshot registry".into()];
        let routes = ["wakes", "registry", "agents", "meta"].map(row).to_vec();
        let (kept, out) = drop_skipped(&o, routes);
        let kept: Vec<&str> = kept.iter().map(|r| r.label.as_str()).collect();
        assert_eq!(
            kept,
            [
                "STREAM /api/p/x/stream snapshot agents",
                "STREAM /api/p/x/stream snapshot meta"
            ]
        );
        assert_eq!(out.len(), 2);
    }
}
