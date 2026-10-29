//! The list of probes: every read the two servers both serve, for the machine and for each
//! scope. Nothing here writes; see [`crate::probe`].

use argus_core::Coll;
use argus_core::ids::PORTFOLIO;

use crate::kind::Kind;
use crate::probe::{McpCall, Probe, Request};

/// The routes that do not depend on a scope.
#[must_use]
pub fn machine() -> Vec<Probe> {
    let get = |kind, path: &str| Probe::get(kind, path);
    vec![
        get(Kind::Health, "/api/health"),
        get(Kind::Version, "/api/version"),
        get(Kind::Projects, "/api/projects"),
        get(Kind::Activity, "/api/activity?limit=1000"),
        get(Kind::Procs, "/api/procs"),
        get(Kind::ProcRules, "/api/procs/rules"),
        get(Kind::Notify, "/api/notify?limit=200"),
        get(Kind::NotifySettings, "/api/notify/settings"),
        get(Kind::Runs, "/api/runs"),
        get(Kind::Presence, "/api/presence"),
        get(Kind::WakeFaults, "/api/wake-faults?all=1"),
        get(Kind::Reconcile, "/api/agents/reconcile"),
        Probe {
            kind: Kind::McpInitialize,
            request: Request::Mcp(McpCall::Initialize),
        },
        Probe {
            kind: Kind::McpTools,
            request: Request::Mcp(McpCall::ToolsList),
        },
    ]
}

/// The routes of one scope: every collection of the db shim that the Node dashboard also has (`prs` is Rust only), and for a project the launches,
/// runs, wakes and open wake streams.
#[must_use]
pub fn scope(key: &str) -> Vec<Probe> {
    let mut out: Vec<Probe> = Coll::ALL
        .iter()
        .filter(|c| **c != Coll::Prs)
        .map(|c| {
            Probe::get(
                Kind::Doc(c.as_str()),
                format!("/api/p/{key}/db/{}", c.as_str()),
            )
        })
        .collect();
    if key != PORTFOLIO {
        out.extend([
            Probe::get(Kind::Launches, format!("/api/p/{key}/launches")),
            Probe::get(Kind::Runs, format!("/api/p/{key}/runs")),
            Probe::get(Kind::Wakes, format!("/api/p/{key}/wakes?limit=200")),
            Probe::get(Kind::Listeners, format!("/api/p/{key}/wakes/listeners")),
        ]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{machine, scope};
    use crate::probe::Request;

    #[test]
    fn every_probe_is_a_get_or_an_mcp_read() {
        for p in machine().into_iter().chain(scope("alpha")) {
            match p.request {
                Request::Get(path) => assert!(path.starts_with("/api/"), "{path}"),
                Request::Mcp(_) => {}
            }
        }
    }

    #[test]
    fn a_project_has_every_collection_and_the_portfolio_has_only_the_collections() {
        assert_eq!(scope("_portfolio").len(), 20);
        assert_eq!(scope("alpha").len(), 24);
    }
}
