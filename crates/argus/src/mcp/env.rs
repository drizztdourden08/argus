//! The machine behind the stdio bridge: `config.json` for the endpoint, `quit.flag` for the
//! owner's Quit, the hidden server start, and stderr for the log.

use argus_core::Paths;
use argus_mcp::{BridgeEnv, Endpoint};
use argus_native::{Boot, TrayFiles, quit_flag_active};

use super::endpoint;
use super::launch::{boot_now, start_hidden};

/// Starts the server: the real start, or a test's.
pub type Launch = Box<dyn Fn() -> Result<(), String> + Send + Sync>;

/// Places this boot in time: the real clocks, or a test's.
pub type Clock = Box<dyn Fn() -> Option<Boot> + Send + Sync>;

/// The bridge's view of one dashboard home.
pub struct DashboardEnv {
    paths: Paths,
    launch: Launch,
    boot: Clock,
}

impl DashboardEnv {
    /// The real machine: the server is started as `argus serve` of this home.
    #[must_use]
    pub fn new(paths: Paths) -> Self {
        let home = paths.home.clone();
        Self::with(
            paths,
            Box::new(move || start_hidden(&home)),
            Box::new(boot_now),
        )
    }

    /// The same home with the start and the boot clock given, so nothing real is started.
    #[must_use]
    pub fn with(paths: Paths, launch: Launch, boot: Clock) -> Self {
        Self {
            paths,
            launch,
            boot,
        }
    }
}

impl BridgeEnv for DashboardEnv {
    fn endpoint(&self) -> Option<Endpoint> {
        endpoint::read(&self.paths.config)
    }

    fn quit_flag_active(&self) -> bool {
        let files = TrayFiles::new(&self.paths.home);
        (self.boot)().is_some_and(|boot| quit_flag_active(&files, boot))
    }

    fn launch(&self) -> Result<(), String> {
        (self.launch)()
    }

    fn log(&self, line: &str) {
        eprintln!("{line}");
    }
}
