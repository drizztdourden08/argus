//! The probe's steps in order.

use std::path::Path;
use std::time::Duration;

use argus_overlord::ports::ApiKey;
use argus_overlord::protocol::{Event, Init, encode_user_message};
use argus_overlord::session::guard::check_init;

use super::child::Child;
use super::probe::Probe;
use super::{cli, turns};

/// How long the child gets to announce itself before the first message is written.
const INIT_WAIT: Duration = Duration::from_secs(20);
/// How long the init may take once the first message is written.
const LATE_INIT_WAIT: Duration = Duration::from_secs(60);
const FIRST: &str = "Reply with exactly the word one.";
const SECOND: &str = "Reply with exactly the word two.";

/// How the steps ended.
pub enum Done {
    Complete,
    /// The billing guard stopped the child.
    Refused,
}

/// Runs steps 1 to 9. The cwd is a fresh empty folder under the OS temp dir.
pub async fn run(probe: &mut Probe, key: &ApiKey, dir: &Path) -> Result<Done, String> {
    let exe = cli::find_exe();
    cli::version(probe, &exe).await?;
    cli::help(probe, &exe).await?;
    let cwd = std::env::temp_dir().join(format!("argus-probe-{}", probe.stamp));
    std::fs::create_dir_all(&cwd).map_err(|e| format!("cannot create the probe cwd: {e}"))?;
    let stderr_log = dir.join(format!("probe-{}.stderr.log", probe.stamp));
    probe.step = "spawn";
    let mut child = Child::start(probe, &exe, key, &cwd, &stderr_log)?;
    let done = converse(probe, &mut child).await;
    let _ignored = std::fs::remove_dir(&cwd);
    done
}

async fn converse(probe: &mut Probe, child: &mut Child) -> Result<Done, String> {
    let early = await_init(probe, child, "init", INIT_WAIT).await?;
    probe.fact(
        "init-before-first-message",
        if early.is_some() { "yes" } else { "no" },
    );
    let sent = early.is_none();
    let init = match early {
        Some(init) => init,
        None => init_after_message(probe, child).await?,
    };
    if let Err(fault) = check_init(&init) {
        probe.kill_child();
        let seen = fault.seen.as_deref().unwrap_or("none");
        probe.fact("billing-guard", &format!("REFUSED {seen}"));
        return Ok(Done::Refused);
    }
    init_facts(probe, &init);
    let one = if sent {
        turns::wait_result(probe, child, "turn1").await?
    } else {
        turns::turn(probe, child, "turn1", FIRST).await?
    };
    let two = turns::turn(probe, child, "turn2", SECOND).await?;
    probe.fact(
        "cost-cumulative",
        turns::cumulative(one.total_cost_usd, two.total_cost_usd),
    );
    probe.step = "exit";
    let exit = child.finish(probe).await;
    probe.fact("exit", &exit);
    Ok(Done::Complete)
}

/// Reads until `system/init`; `None` when `wait` passes first.
async fn await_init(
    probe: &mut Probe,
    child: &mut Child,
    step: &'static str,
    wait: Duration,
) -> Result<Option<Init>, String> {
    probe.step = step;
    let read = async {
        loop {
            if let Event::Init(init) = child.next_event(probe).await? {
                return Ok::<Init, String>(init);
            }
        }
    };
    match tokio::time::timeout(wait, read).await {
        Ok(found) => found.map(Some),
        Err(_) => Ok(None),
    }
}

/// The CLI said nothing until it had a message: send turn 1 and read the init that follows.
async fn init_after_message(probe: &mut Probe, child: &mut Child) -> Result<Init, String> {
    child.send(&encode_user_message(FIRST)).await?;
    await_init(probe, child, "init-after-message", LATE_INIT_WAIT)
        .await?
        .ok_or_else(|| "timeout (60 s) in step init-after-message".to_owned())
}

fn init_facts(probe: &mut Probe, init: &Init) {
    probe.fact(
        "apiKeySource",
        init.api_key_source.as_deref().unwrap_or("none"),
    );
    probe.fact("model", init.model.as_deref().unwrap_or("none"));
    probe.fact("tool-count", &init.tools.len().to_string());
    let servers: Vec<&str> = init
        .mcp_servers
        .iter()
        .map(|s| s.get("name").and_then(|n| n.as_str()).unwrap_or("?"))
        .collect();
    probe.fact("mcp-servers", &servers.join(", "));
    probe.fact("session_id", &init.session_id);
}
