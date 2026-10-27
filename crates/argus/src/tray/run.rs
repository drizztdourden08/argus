//! `argus tray run`: the icon, its menu, and the server behind it.
//!
//! The icon shows the server's state, polled every five seconds. Quit stops the server and then
//! closes the icon; the other items run on the runtime, since the tray thread must return at
//! once. The tray starts the server unless `quit.flag` says the owner closed it during this boot.

use std::sync::Arc;
use std::time::Duration;

use argus_core::Paths;
use argus_native::{
    Boot, Command, TrayError, TrayFiles, TrayHandle, clear_quit_flag, quit_flag_active,
    request_stop,
};
use tokio::runtime::Handle;
use tokio::time::{Instant, interval, sleep_until};

use super::server::Server;
use super::{log, open};

const POLL: Duration = Duration::from_secs(5);
/// How often the server is probed while it is still starting, so the icon turns to running as
/// soon as it answers.
const POLL_STARTING: Duration = Duration::from_millis(500);
const LOOK: Duration = Duration::from_millis(250);

/// What the menu items work with.
struct Menu {
    server: Server,
    files: TrayFiles,
    runtime: Handle,
}

impl Menu {
    fn note(&self, text: &str) {
        log::line(&self.files, text);
    }

    fn report(&self, what: &str, done: Result<(), String>) {
        if let Err(why) = done {
            self.note(&format!("{what}: {why}"));
        }
    }

    async fn start(&self) {
        self.report("start", self.server.start().await);
    }

    /// Opens a page of the dashboard. Opening it is how the owner reopens a closed dashboard,
    /// so `quit.flag` goes and a stopped server starts.
    async fn show(&self, next: &str) {
        let _cleared = clear_quit_flag(&self.files);
        self.start().await;
        let Some(address) = self.server.page(next).await else {
            return self.note("open: the server did not answer");
        };
        self.report("open", open::url(&address).map_err(|e| e.to_string()));
    }

    async fn restart(&self) {
        self.server.stop().await;
        let _cleared = clear_quit_flag(&self.files);
        self.start().await;
    }

    async fn quit(&self) {
        self.server.stop().await;
        self.report(
            "close",
            request_stop(&self.files).map_err(|e| e.to_string()),
        );
    }

    async fn run(&self, command: Command) {
        match command {
            Command::Open => self.show("/").await,
            Command::OpenNotifications => self.show("/#notifications").await,
            Command::OpenLogs => {
                let done = open::folder(&self.files.log_dir()).map_err(|e| e.to_string());
                self.report("open logs", done);
            }
            Command::Restart => self.restart().await,
            Command::Start => {
                let _cleared = clear_quit_flag(&self.files);
                self.start().await;
            }
            Command::Quit => self.quit().await,
        }
    }
}

fn wire(tray: &TrayHandle, menu: &Arc<Menu>) {
    let quitting = Arc::clone(menu);
    tray.on_quit(move || {
        let menu = Arc::clone(&quitting);
        drop(quitting.runtime.spawn(async move { menu.quit().await }));
    });
    let clicked = Arc::clone(menu);
    tray.on_command(move |command| {
        let menu = Arc::clone(&clicked);
        drop(
            clicked
                .runtime
                .spawn(async move { menu.run(command).await }),
        );
    });
}

/// Probes the server at once, then every few seconds (faster while it starts), until the icon is
/// closed.
async fn watch(tray: &TrayHandle, menu: &Menu) {
    let mut look = interval(LOOK);
    let mut due = Instant::now();
    loop {
        tokio::select! {
            () = sleep_until(due) => {
                tray.set_state(menu.server.state().await);
                let wait = if menu.server.is_starting() { POLL_STARTING } else { POLL };
                due = Instant::now() + wait;
            }
            _ = look.tick() => {
                if !tray.is_running() {
                    return;
                }
            }
        }
    }
}

/// Runs the tray until it is closed.
///
/// # Errors
///
/// The icon could not be shown.
pub async fn run(paths: Paths) -> Result<u8, String> {
    let files = TrayFiles::new(&paths.home);
    let tray = match TrayHandle::start(files.clone()) {
        Ok(tray) => tray,
        Err(TrayError::AlreadyRunning) => {
            println!("tray already running");
            return Ok(0);
        }
        Err(error) => return Err(error.to_string()),
    };
    let menu = Arc::new(Menu {
        server: Server::new(paths),
        files,
        runtime: Handle::current(),
    });
    wire(&tray, &menu);
    if !quit_flag_active(&menu.files, Boot::current()) {
        menu.server.expect_start();
        let booting = Arc::clone(&menu);
        drop(menu.runtime.spawn(async move { booting.start().await }));
    }
    watch(&tray, &menu).await;
    let _reason = tray.wait();
    Ok(0)
}
