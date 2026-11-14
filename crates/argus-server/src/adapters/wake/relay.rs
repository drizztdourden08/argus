//! The server's side of the wake relay: the relay note as an inbox line from `Argus`, the headless resume
//! and the owner notice. Every call returns at once: the work runs on its own thread, off the service lock.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use argus_accounts::session_map::{local_ids, seat_of};
use argus_accounts::find_cli_exe;
use argus_board::inbox::queue;
use argus_board::store::with_board;
use argus_notify::{By, Level, Notify, PingRequest};
use argus_wake::relay::{RelayPort, ThreadKey, Ward};

use super::headless::{resume, session_running};
use super::relay_text::{Line, note_text, wake_text};

/// What the relay port reaches.
pub struct RelayHost {
    /// Where the boards live.
    pub coord_home: PathBuf,
    /// A shadow run writes nothing.
    pub shadow: bool,
    /// The desktop app's data folder, looked up when it is needed.
    pub app_data: Box<dyn Fn() -> Option<PathBuf> + Send + Sync>,
    /// The owner notifications.
    pub notify: Option<Arc<Notify>>,
}

/// The relay port over the boards, the desktop's session records and the CLI.
pub struct ServerRelay {
    host: Arc<RelayHost>,
}

impl ServerRelay {
    /// A port over the host.
    #[must_use]
    pub fn new(host: RelayHost) -> Self {
        Self {
            host: Arc::new(host),
        }
    }
}

impl RelayHost {
    fn post_note(&self, via: &ThreadKey, targets: &[Ward]) {
        let Some(app) = (self.app_data)() else {
            return;
        };
        let wanted: Vec<String> = targets.iter().filter_map(|t| t.session_id.clone()).collect();
        let local = local_ids(&app, &wanted);
        let lines: Vec<Line<'_>> = targets.iter().filter_map(|t| line_of(t, &local)).collect();
        if lines.is_empty() {
            return;
        }
        let text = note_text(&lines);
        drop(with_board(&self.coord_home, &via.project, false, |board| {
            Ok(queue(board, &via.thread, "Argus", "note", &text, None))
        }));
    }

    fn resume_headless(&self, target: &Ward) {
        let Some(session) = target.session_id.as_deref() else {
            return;
        };
        let Some(app) = (self.app_data)() else {
            return;
        };
        let Some(exe) = find_cli_exe(&app) else {
            return;
        };
        let (id, cwd) = seat_for(&app, session);
        let Some(cwd) = cwd else {
            return;
        };
        if !session_running(&id) && !session_running(session) {
            drop(resume(
                &exe,
                Path::new(&cwd),
                &id,
                &wake_text(&target.key.thread),
            ));
        }
    }

    fn tell_owner(&self, target: &Ward) {
        let Some(notify) = &self.notify else {
            return;
        };
        drop(notify.send(PingRequest {
            title: Some(format!("{} is asleep", target.key.thread)),
            text: format!(
                "{} {} holds a wake and could not be woken. Open its session to wake it.",
                target.key.project, target.key.thread
            ),
            level: Level::Ping,
            project: Some(target.key.project.clone()),
            thread: Some(target.key.thread.clone()),
            by: By::Auto,
            url: None,
        }));
    }
}

fn line_of<'a>(
    target: &'a Ward,
    local: &'a std::collections::HashMap<String, String>,
) -> Option<Line<'a>> {
    let id = local.get(target.session_id.as_ref()?)?;
    Some(Line {
        project: &target.key.project,
        thread: &target.key.thread,
        local_id: id,
    })
}

/// The CLI session id the desktop resumes now and the folder of the session the hook's id belongs to.
fn seat_for(app: &Path, session: &str) -> (String, Option<String>) {
    let wanted = [session.to_owned()];
    let seat = local_ids(app, &wanted)
        .get(session)
        .and_then(|local| seat_of(app, local));
    let id = seat
        .as_ref()
        .and_then(|s| s.cli_session_id.clone())
        .unwrap_or_else(|| session.to_owned());
    (id, seat.and_then(|s| s.cwd))
}

impl RelayPort for ServerRelay {
    fn note(&self, via: &ThreadKey, targets: &[Ward]) {
        if self.host.shadow {
            return;
        }
        let (host, via, targets) = (Arc::clone(&self.host), via.clone(), targets.to_vec());
        drop(thread::spawn(move || host.post_note(&via, &targets)));
    }

    fn headless(&self, target: &Ward) {
        if self.host.shadow {
            return;
        }
        let (host, target) = (Arc::clone(&self.host), target.clone());
        drop(thread::spawn(move || host.resume_headless(&target)));
    }

    fn notice(&self, target: &Ward) {
        if self.host.shadow {
            return;
        }
        let (host, target) = (Arc::clone(&self.host), target.clone());
        drop(thread::spawn(move || host.tell_owner(&target)));
    }
}
