//! What the menu items and a click on the icon do. They run on the runtime, since the tray thread
//! must return at once, and every way they can fail writes one line to `logs/tray.log`: the tray
//! has no console, so a silent failure would leave the owner with a click that does nothing.

use std::future::Future;
use std::sync::Arc;

use argus_native::{Command, TrayFiles, clear_quit_flag, request_stop};
use tokio::runtime::Handle;
use tokio::task::{JoinHandle, block_in_place};

use super::server::Server;
use super::{log, open};

/// Opens an address in the browser, or says why it could not. It may block for a moment.
pub type Opener = Box<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

/// What the menu items work with.
pub struct Menu {
    pub server: Server,
    pub files: TrayFiles,
    pub runtime: Handle,
    pub opener: Opener,
}

/// The address without its query, so a one-time sign-in code never lands in the log.
fn shown(address: &str) -> &str {
    address.split('?').next().unwrap_or(address)
}

impl Menu {
    pub fn note(&self, text: &str) {
        log::line(&self.files, text);
    }

    fn report(&self, what: &str, done: Result<(), String>) {
        if let Err(why) = done {
            self.note(&format!("{what}: {why}"));
        }
    }

    /// Runs `work` on the runtime; a task that panics says so in the log. Answers the watch,
    /// which ends once `work` has.
    pub fn spawn(
        self: &Arc<Self>,
        what: &str,
        work: impl Future<Output = ()> + Send + 'static,
    ) -> JoinHandle<()> {
        let task = self.runtime.spawn(work);
        let menu = Arc::clone(self);
        let what = what.to_owned();
        self.runtime.spawn(async move {
            if let Err(error) = task.await {
                menu.note(&format!("{what}: the task failed: {error}"));
            }
        })
    }

    pub async fn start(&self) {
        self.report("start", self.server.start().await);
    }

    /// Opens a page of the dashboard. Opening it is how the owner reopens a closed dashboard,
    /// so `quit.flag` goes and a stopped server starts.
    async fn show(&self, what: &str, next: &str) {
        let _cleared = clear_quit_flag(&self.files);
        self.start().await;
        match self.server.page(next).await {
            Ok(address) => {
                self.note(&format!("{what}: {}", shown(&address)));
                let opened = block_in_place(|| (self.opener)(&address));
                self.report(what, opened);
            }
            Err(why) => self.note(&format!("{what}: {why}")),
        }
    }

    async fn restart(&self) {
        self.server.stop().await;
        let _cleared = clear_quit_flag(&self.files);
        self.start().await;
    }

    pub async fn quit(&self) {
        self.server.stop().await;
        self.report(
            "close",
            request_stop(&self.files).map_err(|e| e.to_string()),
        );
    }

    pub async fn run(&self, command: Command) {
        let what = command.id();
        match command {
            Command::Open => self.show(what, "/").await,
            Command::OpenNotifications => self.show(what, "/#notifications").await,
            Command::OpenLogs => {
                let done = open::folder(&self.files.log_dir()).map_err(|e| e.to_string());
                self.report(what, done);
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

// test-tag: standard
#[cfg(test)]
mod tests {
    //! S2: the tray menu commands against an in-process server.
    use std::fs;
    use std::sync::{Arc, Mutex, PoisonError};

    use argus_core::Paths;
    use argus_native::{Command, TrayFiles};
    use argus_testkit::{TestOptions, TestServer};
    use serde_json::{Value, json};
    use tokio::runtime::Handle;

    use super::{Menu, Opener};
    use crate::tray::server::Server;

    type Seen = Arc<Mutex<Vec<String>>>;

    fn recording(seen: &Seen, answer: Result<(), String>) -> Opener {
        let seen = Arc::clone(seen);
        Box::new(move |address| {
            seen.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(address.to_owned());
            answer.clone()
        })
    }

    fn menu(test: &TestServer, opener: Opener) -> Menu {
        let paths = Paths::new(&test.coord_home, Some(test.dashboard_home.clone()));
        Menu {
            files: TrayFiles::new(&paths.home),
            server: Server::new(paths),
            runtime: Handle::current(),
            opener,
        }
    }

    fn log_of(menu: &Menu) -> String {
        fs::read_to_string(menu.files.log_file()).unwrap_or_default()
    }

    async fn server(config: Value) -> TestServer {
        let options = TestOptions {
            config,
            ..TestOptions::default()
        };
        TestServer::start(options).await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn open_and_notifications_open_the_page_and_log_the_address() {
        let test = server(json!({})).await;
        let seen = Seen::default();
        let menu = menu(&test, recording(&seen, Ok(())));
        menu.run(Command::Open).await;
        menu.run(Command::OpenNotifications).await;
        let base = format!("http://127.0.0.1:{}", test.port);
        let opened = seen.lock().unwrap_or_else(PoisonError::into_inner).clone();
        assert_eq!(
            opened,
            [format!("{base}/"), format!("{base}/#notifications")]
        );
        assert!(log_of(&menu).contains(&format!("open: {base}/")));
        test.stop(false).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn an_opener_that_fails_says_why_in_the_log() {
        let test = server(json!({})).await;
        let seen = Seen::default();
        let menu = menu(&test, recording(&seen, Err("no browser".to_owned())));
        menu.run(Command::Open).await;
        assert!(log_of(&menu).contains("open: no browser"));
        test.stop(false).await;
    }

    /// The local names on a loopback address of the test's own, every socket on port 0.
    async fn named_server() -> TestServer {
        let options = TestOptions {
            config: json!({ "localNames": { "enabled": true, "address": "127.65.82.73" } }),
            tweak: Some(Arc::new(|o| {
                o.local_names.dns_port = 0;
                o.local_names.door_port = 0;
            })),
            ..TestOptions::default()
        };
        TestServer::start(options).await
    }

    /// The `.argus` names need browser setup, so the tray never prefers them: with them up and no
    /// localhost door, it still opens `127.0.0.1`.
    #[tokio::test(flavor = "multi_thread")]
    async fn with_the_argus_names_up_the_tray_still_opens_the_loopback_page() {
        let test = named_server().await;
        assert!(test.svc.names().is_some(), "the names are up");
        let seen = Seen::default();
        let menu = menu(&test, recording(&seen, Ok(())));
        menu.run(Command::Open).await;
        let opened = seen.lock().unwrap_or_else(PoisonError::into_inner).clone();
        assert_eq!(opened, [format!("http://127.0.0.1:{}/", test.port)]);
        test.stop(false).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_task_that_panics_says_so_in_the_log() {
        let test = server(json!({})).await;
        let menu = Arc::new(menu(&test, recording(&Seen::default(), Ok(()))));
        let watched = menu.spawn("open", async { panic!("boom") });
        watched.await.expect("the watcher ends");
        assert!(log_of(&menu).contains("open: the task failed"));
        test.stop(false).await;
    }
}
