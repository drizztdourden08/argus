//! `argus dns`: the local names of Argus. `status` reads how they stand; `install --print` and
//! `remove --print` print the one-time administrator PowerShell the owner runs himself. Nothing
//! here runs that script or changes a system setting.

mod script;
mod status;

use argus_core::Paths;
use argus_core::config::{env_port, peek_config};
use argus_core::names::{self, LocalNames};
use clap::Subcommand;

use super::{Home, paths_of};

/// What to do with the local names.
#[derive(Subcommand)]
pub enum Action {
    /// Whether the names are on, whether the DNS answers and where the front door is.
    Status {
        #[command(flatten)]
        home: Home,
    },
    /// Prints the one-time administrator PowerShell that points `.<suffix>` at Argus.
    Install {
        /// Required: the script is printed for the owner to run, never run by Argus.
        #[arg(long, required = true)]
        print: bool,
        #[command(flatten)]
        home: Home,
    },
    /// Prints the administrator PowerShell that undoes `install`.
    Remove {
        /// Required: the script is printed for the owner to run, never run by Argus.
        #[arg(long, required = true)]
        print: bool,
        #[command(flatten)]
        home: Home,
    },
}

/// The `localNames` of the home's config, read without creating the file; the defaults when the
/// file or the key is missing.
fn names_of(paths: &Paths) -> Result<LocalNames, String> {
    let config = peek_config(paths, env_port().ok().flatten());
    match config.as_ref().and_then(|c| c.get(names::KEY)) {
        Some(value) => LocalNames::parse(value),
        None => Ok(LocalNames::default()),
    }
}

/// Does what the action asks and answers the exit code.
///
/// # Errors
///
/// The `localNames` key of the config is refused.
pub async fn run(action: Action) -> Result<u8, String> {
    match action {
        Action::Status { home } => {
            let paths = paths_of(home.data_dir);
            Ok(status::run(&paths, names_of(&paths)?).await)
        }
        Action::Install { home, .. } => {
            print!("{}", script::install(&names_of(&paths_of(home.data_dir))?));
            Ok(0)
        }
        Action::Remove { home, .. } => {
            print!("{}", script::remove(&names_of(&paths_of(home.data_dir))?));
            Ok(0)
        }
    }
}
