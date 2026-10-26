//! The process host of the run service: the machine's, or in tests the fake one.

use std::io;
use std::time::Duration;

use argus_launchers::{FakeHost, ProcessHost, SpawnRequest, SpawnedProcess, SystemHost};

/// Where the runs' processes come from.
#[derive(Clone)]
pub enum RunHost {
    /// Real processes.
    System(SystemHost),
    /// Pipes a test drives by hand.
    Fake(FakeHost),
}

impl ProcessHost for RunHost {
    fn spawn(&self, request: &SpawnRequest) -> io::Result<SpawnedProcess> {
        match self {
            RunHost::System(host) => host.spawn(request),
            RunHost::Fake(host) => host.spawn(request),
        }
    }

    async fn kill_tree(&self, pid: u32, grace: Duration) {
        match self {
            RunHost::System(host) => host.kill_tree(pid, grace).await,
            RunHost::Fake(host) => host.kill_tree(pid, grace).await,
        }
    }

    fn is_alive(&self, pid: u32) -> bool {
        match self {
            RunHost::System(host) => host.is_alive(pid),
            RunHost::Fake(host) => host.is_alive(pid),
        }
    }

    async fn looks_like_ours(&self, pid: u32) -> bool {
        match self {
            RunHost::System(host) => host.looks_like_ours(pid).await,
            RunHost::Fake(host) => host.looks_like_ours(pid).await,
        }
    }
}
