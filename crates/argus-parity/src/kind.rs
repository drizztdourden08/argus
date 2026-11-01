//! The kinds of answer and the volatile fields of each. These tables are the normalisation
//! rules of the comparison; every entry is a field the Node server and Argus cannot be expected
//! to agree on by value, or an array whose order neither promises.
//!
//! Always forgiven, on every route: object key order, `seq` numbers (each server counts its
//! own), integer versus float spelling of a number, and timestamps within the tolerance
//! (`--tolerance-ms`, 2 s by default): two strings that both parse as a date and time, or two
//! epoch numbers under a timestamp key (see `diff`).
//!
//! Set aside by kind, because they depend on who is asked and when: the `live` sub-object of an
//! agent (a fresh server only tracks agents that ended within `recentMs`, so a finished agent
//! loses it at any restart), and the listener-derived fields of a registry doc (`listening`,
//! `listenerLabel`, `state`, `fault`, `since`, `deaf`, `boardOpen`, `boardSince`,
//! `boardRequired`), which follow who listens to each server.
// test-tag: critical

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
/// far, and a fresh server only tracks agents that ended within `recentMs`, so an agent that has
/// finished loses its `live` at any restart, Node included. The whole sub-object is set aside, so
/// a `live` that exists on one side only is equal.
static DOC_AGENTS: Spec = Spec {
    ignore: &["seq", "docs.*.data.live"],
    unordered: &[("docs", "id")],
    drop_rows: &[],
};

/// The thread registry. The fields that say who listens to a server (the listener, its label, the
/// state and fault derived from it, the board-open flags) belong to the process that is asked, so
/// two servers with different listeners cannot agree on them.
static DOC_REGISTRY: Spec = Spec {
    ignore: &[
        "seq",
        "docs.*.data.listening",
        "docs.*.data.listenerLabel",
        "docs.*.data.state",
        "docs.*.data.fault",
        "docs.*.data.since",
        "docs.*.data.deaf",
        "docs.*.data.boardOpen",
        "docs.*.data.boardSince",
        "docs.*.data.boardRequired",
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
        "registry" => &DOC_REGISTRY,
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
    fn the_live_state_of_an_agent_is_never_compared_but_the_rest_is() {
        let before = json!({ "turns": 3, "tokens": { "input": 1 }, "modelUsed": "opus" });
        let later = json!({ "turns": 5, "tokens": { "input": 4 }, "modelUsed": "haiku" });
        let rules = Kind::Doc("agents").rules(0);
        assert_eq!(
            judge(rules, &agents(&before), &agents(&later)),
            Verdict::Equal
        );
        let named =
            |name: &str| Got::Doc(json!({ "docs": [{ "id": "A-1", "data": { "name": name } }] }));
        let Verdict::Differs(c) = judge(rules, &named("x"), &named("y")) else {
            panic!("the name differs");
        };
        assert_eq!(c[0].path, "docs[A-1].data.name");
    }

    #[test]
    fn a_live_that_exists_on_one_side_only_is_equal() {
        let rules = Kind::Doc("agents").rules(0);
        let gone = Got::Doc(json!({ "docs": [{ "id": "A-1", "data": { "name": "x" } }] }));
        let live = json!({ "turns": 3, "modelUsed": "opus" });
        assert_eq!(judge(rules, &agents(&live), &gone), Verdict::Equal);
        assert_eq!(judge(rules, &gone, &agents(&live)), Verdict::Equal);
    }

    fn registry(listening: bool, state: &str, role: &str) -> Got {
        Got::Doc(json!({ "docs": [{ "id": "web", "data": {
            "role": role, "listening": listening, "listenerLabel": state, "state": state,
            "fault": state, "since": state, "deaf": listening, "boardOpen": listening,
            "boardSince": state, "boardRequired": listening,
        } }] }))
    }

    #[test]
    fn the_listener_fields_of_the_registry_are_never_compared_but_the_rest_is() {
        let rules = Kind::Doc("registry").rules(0);
        let (a, b) = (
            registry(true, "live", "lead"),
            registry(false, "gone", "lead"),
        );
        assert_eq!(judge(rules, &a, &b), Verdict::Equal);
        let Verdict::Differs(c) = judge(rules, &a, &registry(true, "live", "ui")) else {
            panic!("the role differs");
        };
        assert_eq!(c[0].path, "docs[web].data.role");
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
}
