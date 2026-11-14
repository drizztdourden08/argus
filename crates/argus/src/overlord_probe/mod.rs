//! `argus overlord probe`: a hidden one-shot that checks facts about the real `claude` CLI on the
//! stored API key (story 4 of I-159). The key goes only into the child env; every line that
//! leaves this module has it replaced by `<redacted>` first.

mod child;
mod cli;
mod probe;
mod redact;
mod steps;
mod turns;

use std::path::{Path, PathBuf};
use std::time::Duration;

use argus_native::KeyStore;
use argus_overlord::ports::ApiKey;
use clap::Subcommand;
use tokio::time::error::Elapsed;

use crate::lifecycle;
use probe::Probe;
use steps::Done;

/// The whole probe may take this long.
const OVERALL: Duration = Duration::from_secs(300);

/// The overlord verbs.
#[derive(Subcommand)]
pub enum Verb {
    /// Checks facts about the real `claude` CLI on the stored API key (a few cents).
    Probe {
        /// The dashboard home; default `DASHBOARD_HOME`, else `<COORD_HOME>/dashboard`.
        #[arg(long, alias = "home")]
        data_dir: Option<PathBuf>,
    },
}

/// Runs the verb.
///
/// # Errors
/// No key is stored, or a file of the report cannot be written.
pub async fn run(verb: Verb) -> Result<u8, String> {
    let Verb::Probe { data_dir } = verb;
    let home = lifecycle::paths_of(data_dir).home;
    let key = load_key(&home)?;
    let dir = home.join("overlord");
    let mut probe = Probe::open(&dir, key.expose())?;
    let outcome = tokio::time::timeout(OVERALL, steps::run(&mut probe, &key, &dir)).await;
    let code = settle(&mut probe, outcome);
    probe.save_facts(&dir)?;
    Ok(code)
}

fn load_key(home: &Path) -> Result<ApiKey, String> {
    match KeyStore::new(home).load() {
        Ok(Some(secret)) => Ok(ApiKey::new(secret.expose())),
        Ok(None) => Err("no API key is stored; save one in the Argus settings first".to_owned()),
        Err(e) => Err(format!("cannot read the stored key: {e}")),
    }
}

/// Turns the end of the steps into an exit code, killing a child that is still there.
fn settle(probe: &mut Probe, outcome: Result<Result<Done, String>, Elapsed>) -> u8 {
    let code = match outcome {
        Ok(Ok(Done::Complete)) => 0,
        Ok(Ok(Done::Refused)) => 2,
        Ok(Err(message)) => {
            probe.fact("error", &message);
            1
        }
        Err(_) => {
            let step = probe.step;
            probe.fact("timeout", &format!("overall 300 s, in step {step}"));
            1
        }
    };
    probe.kill_child();
    code
}
