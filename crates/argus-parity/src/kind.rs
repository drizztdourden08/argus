//! The kinds of answer and the volatile fields of each. These tables are the normalisation
//! rules of the comparison; every entry is a field the Node server and Argus cannot be expected
//! to agree on by value, or an array whose order neither promises.
//!
//! Always forgiven, on every route: object key order, `seq` numbers (each server counts its
//! own), integer versus float spelling of a number, and timestamps within the tolerance
//! (`--tolerance-ms`, 2 s by default): two strings that both parse as a date and time, or two
//! epoch numbers under a timestamp key (see `diff`).

use crate::rules::{RowDrop, Rules, Spec};

/// The kind of an answer, which picks its rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `/api/health`.
    Health,
    /// `/api/version`.
    Version,
    /// `/api/projects`.
    Projects,
    /// `/api/activity`.
    Activity,
    /// `/api/procs`.
    Procs,
    /// `/api/procs/rules`.
    ProcRules,
    /// `/api/notify`.
    Notify,
    /// `/api/notify/settings`.
    NotifySettings,
    /// A list of runs.
    Runs,
    /// The launch defs of a project.
    Launches,
    /// The wakes of a project.
    Wakes,
    /// The open wake streams of a project.
    Listeners,
    /// `/api/presence`.
    Presence,
    /// `/api/wake-faults`.
    WakeFaults,
    /// `/api/agents/reconcile`.
    Reconcile,
    /// MCP `initialize`.
    McpInitialize,
    /// MCP `tools/list`.
    McpTools,
    /// The docs of one collection, from the db route or a stream snapshot.
    Doc(&'static str),
    /// The `ready` frame of a doc stream.
    Ready,
}

/// Rows of a server's own life: the dashboard's start and stop lines are per instance.
const LIFECYCLE: &[&str] = &["dashboard started", "dashboard stopped"];

static NONE: Spec = Spec {
    ignore: &[],
    unordered: &[],
    drop_rows: &[],
};

static DOC: Spec = Spec {
    ignore: &["seq"],
    unordered: &[("docs", "id")],
    drop_rows: &[],
};

static DOC_PROCS: Spec = Spec {
    ignore: &[
        "seq",
        "docs.*.data.memMB",
        "docs.*.data.cpuPct",
        "docs.*.data.ageMin",
        "docs.*.data.idleMin",
        "docs.*.data.since",
    ],
    unordered: &[("docs", "id")],
    drop_rows: &[],
};

static DOC_META: Spec = Spec {
    ignore: &[
        "seq",
        "docs.*.data.sampledAt",
        "docs.*.data.sampleMs",
        "docs.*.data.tookMs",
        "docs.*.data.totalMemMB",
        "docs.*.data.rulesFile",
        "docs.*.data.byRule.*.memMB",
    ],
    unordered: &[("docs", "id")],
    drop_rows: &[],
};

/// The watcher's `live` state of an agent counts what each server has read of the transcript so
/// far, so two servers sampling at different moments cannot agree on it.
static DOC_AGENTS: Spec = Spec {
    ignore: &[
        "seq",
        "docs.*.data.live.turns",
        "docs.*.data.live.toolCalls",
        "docs.*.data.live.tokens",
        "docs.*.data.live.activity",
        "docs.*.data.live.activitySince",
        "docs.*.data.live.lastActivityAt",
        "docs.*.data.live.lastMessage",
        "docs.*.data.live.lastTool",
        "docs.*.data.live.stalled",
    ],
    unordered: &[("docs", "id")],
    drop_rows: &[],
};

static DOC_ACTIVITY: Spec = Spec {
    ignore: &["seq", "docs.*.data.n"],
    unordered: &[("docs", "id")],
    drop_rows: &[RowDrop {
        array: "docs",
        field: "data.text",
        prefixes: LIFECYCLE,
    }],
};

static DOC_NOTIFY: Spec = Spec {
    ignore: &["seq", "docs.*.data.toast", "docs.*.data.why"],
    unordered: &[("docs", "id")],
    drop_rows: &[],
};

static HEALTH: Spec = Spec {
    ignore: &[
        "instance",
        "startedAt",
        "port",
        "impl",
        "shadow",
        "version",
        "sse",
        "watcher.lastEventAt",
    ],
    unordered: &[("scopes", "")],
    drop_rows: &[],
};

static VERSION: Spec = Spec {
    ignore: &["version", "rust", "node", "pages"],
    unordered: &[],
    drop_rows: &[],
};

static PROJECTS: Spec = Spec {
    ignore: &[],
    unordered: &[("", "key")],
    drop_rows: &[],
};

static ACTIVITY: Spec = Spec {
    ignore: &["activity.*.id", "activity.*.n"],
    unordered: &[("activity", "")],
    drop_rows: &[RowDrop {
        array: "activity",
        field: "text",
        prefixes: LIFECYCLE,
    }],
};

static PROCS: Spec = Spec {
    ignore: &[
        "meta.sampledAt",
        "meta.sampleMs",
        "meta.tookMs",
        "meta.totalMemMB",
        "meta.rulesFile",
        "meta.byRule.*.memMB",
        "procs.*.memMB",
        "procs.*.cpuPct",
        "procs.*.ageMin",
        "procs.*.idleMin",
        "procs.*.since",
    ],
    unordered: &[("procs", "pid"), ("meta.byRule", "rule")],
    drop_rows: &[],
};

static PROC_RULES: Spec = Spec {
    ignore: &["file"],
    unordered: &[("rules", "name"), ("protectedNames", "")],
    drop_rows: &[],
};

static NOTIFY: Spec = Spec {
    ignore: &["notifications.*.toast", "notifications.*.why"],
    unordered: &[("notifications", "id")],
    drop_rows: &[],
};

static RUNS: Spec = Spec {
    ignore: &[],
    unordered: &[("runs", "id")],
    drop_rows: &[],
};

static LAUNCHES: Spec = Spec {
    ignore: &[],
    unordered: &[("defs", "name")],
    drop_rows: &[],
};

static WAKES: Spec = Spec {
    ignore: &[],
    unordered: &[("wakes", "id")],
    drop_rows: &[],
};

static LISTENERS: Spec = Spec {
    ignore: &["*.since"],
    unordered: &[("", "")],
    drop_rows: &[],
};

static PRESENCE: Spec = Spec {
    ignore: &["open.*.since", "open.*.at"],
    unordered: &[("open", "")],
    drop_rows: &[],
};

static WAKE_FAULTS: Spec = Spec {
    ignore: &[],
    unordered: &[("faults", "")],
    drop_rows: &[],
};

/// The counts of a pass follow which agents are alive or quiet at the moment each server looked.
static RECONCILE: Spec = Spec {
    ignore: &["at", "ms", "projects.*.*", "findings.*.detail"],
    unordered: &[("findings", "")],
    drop_rows: &[],
};

static MCP_INITIALIZE: Spec = Spec {
    ignore: &["id", "result.serverInfo.version"],
    unordered: &[],
    drop_rows: &[],
};

static MCP_TOOLS: Spec = Spec {
    ignore: &["id"],
    unordered: &[("result.tools", "name")],
    drop_rows: &[],
};

fn doc_spec(coll: &str) -> &'static Spec {
    match coll {
        "procs" => &DOC_PROCS,
        "meta" => &DOC_META,
        "agents" => &DOC_AGENTS,
        "activity" => &DOC_ACTIVITY,
        "notifications" => &DOC_NOTIFY,
        _ => &DOC,
    }
}

impl Kind {
    /// The static rules of this kind.
    #[must_use]
    pub fn spec(self) -> &'static Spec {
        match self {
            Kind::Health => &HEALTH,
            Kind::Version => &VERSION,
            Kind::Projects => &PROJECTS,
            Kind::Activity => &ACTIVITY,
            Kind::Procs => &PROCS,
            Kind::ProcRules => &PROC_RULES,
            Kind::Notify => &NOTIFY,
            Kind::NotifySettings | Kind::Ready => &NONE,
            Kind::Runs => &RUNS,
            Kind::Launches => &LAUNCHES,
            Kind::Wakes => &WAKES,
            Kind::Listeners => &LISTENERS,
            Kind::Presence => &PRESENCE,
            Kind::WakeFaults => &WAKE_FAULTS,
            Kind::Reconcile => &RECONCILE,
            Kind::McpInitialize => &MCP_INITIALIZE,
            Kind::McpTools => &MCP_TOOLS,
            Kind::Doc(coll) => doc_spec(coll),
        }
    }

    /// The rules of this kind with the given timestamp tolerance.
    #[must_use]
    pub fn rules(self, tolerance_ms: i64) -> Rules {
        Rules {
            spec: self.spec(),
            tolerance_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::Kind;
    use crate::got::Got;
    use crate::verdict::{Verdict, judge};

    fn agents(live: &Value) -> Got {
        Got::Doc(json!({ "docs": [{ "id": "A-1", "data": { "name": "x", "live": live } }] }))
    }

    fn pass(alive: u64, class: &str, detail: &str) -> Got {
        Got::Doc(json!({
            "at": "t",
            "ms": alive,
            "projects": { "p": { "alive": alive, "quiet": 1 } },
            "findings": [{ "id": "A-1", "class": class, "detail": detail }],
        }))
    }

    #[test]
    fn the_live_agent_numbers_are_never_compared_but_the_model_is() {
        let before = json!({ "turns": 3, "toolCalls": 9, "tokens": { "input": 1 }, "activitySince": "2026-01-01T00:00:00Z", "lastMessage": "a", "modelUsed": "opus" });
        let later = json!({ "turns": 5, "toolCalls": 12, "tokens": { "input": 4 }, "activitySince": "2026-01-01T00:09:00Z", "lastMessage": "b", "modelUsed": "opus" });
        let rules = Kind::Doc("agents").rules(0);
        let same = judge(rules, &agents(&before), &agents(&later));
        assert_eq!(same, Verdict::Equal);
        let other = json!({ "turns": 3, "modelUsed": "haiku" });
        let Verdict::Differs(c) = judge(rules, &agents(&before), &agents(&other)) else {
            panic!("the model differs");
        };
        assert_eq!(c[0].path, "docs[A-1].data.live.modelUsed");
    }

    #[test]
    fn the_reconcile_counts_and_timings_are_never_compared_but_the_classes_are() {
        let rules = Kind::Reconcile.rules(0);
        let same = judge(
            rules,
            &pass(1, "quiet", "grew 3s ago"),
            &pass(4, "quiet", "grew 9s ago"),
        );
        assert_eq!(same, Verdict::Equal);
        let other = judge(rules, &pass(1, "quiet", "x"), &pass(1, "idle", "x"));
        assert!(matches!(other, Verdict::Differs(_)));
    }

    #[test]
    fn every_doc_kind_ignores_seq() {
        for coll in ["items", "procs", "meta", "activity", "notifications"] {
            assert!(Kind::Doc(coll).spec().ignore.contains(&"seq"), "{coll}");
        }
    }

    #[test]
    fn the_process_sample_numbers_are_never_compared() {
        assert!(Kind::Procs.spec().ignore.contains(&"procs.*.memMB"));
    }
}
