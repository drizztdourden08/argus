//! The notifications to the owner: a [`Notify`] over the table, the live config and the OS toast
//! with the tray balloon behind it. `svc.notifier` is built first, the service needs it. Each
//! project board export asks the service for the automatic pings, and a hard resync of the
//! portfolio scope writes the docs again.

use std::sync::Arc;
use std::sync::mpsc::{self, Sender};
use std::thread;

use argus_board::names::MAIN_KEY;
use argus_core::clock::{Clock, SystemClock};
use argus_core::ids::PORTFOLIO;
use argus_native::{Notifier, NotifyError, RecordingNotifier, Toast};
use argus_notify::{BoardFacts, Delivery, Notify, Ports, TrayFallback};

use crate::adapters::activity::ActivityAdapter;
use crate::adapters::notify::{
    BoardNames, DocStream, LiveConfig, NotifyDocs, TrayBalloon, facts_of,
};
use crate::boards::BoardReader;
use crate::{ServeOptions, Services};

/// How the notifications start; the fields after `enabled` choose the toast and are seams for
/// tests. Without `system` no real toast is ever shown.
#[derive(Clone)]
pub struct NotifyOptions {
    /// Run the notifications.
    pub enabled: bool,
    /// Show toasts through the operating system. A shadow run never does.
    pub system: bool,
    /// Another way to show a toast than the system's.
    pub notifier: Option<Arc<dyn Notifier>>,
    /// Another way to show a balloon than the tray's, when a toast fails.
    pub fallback: Option<Arc<dyn TrayFallback>>,
    /// Another clock than the system's, for the limits.
    pub clock: Option<Arc<dyn Clock>>,
}

impl Default for NotifyOptions {
    fn default() -> Self {
        NotifyOptions {
            enabled: true,
            system: false,
            notifier: None,
            fallback: None,
            clock: None,
        }
    }
}

/// A toast that cannot be shown here, so every notification is recorded as `failed`.
struct Refusing(NotifyError);

impl Notifier for Refusing {
    fn show(&self, _toast: &Toast) -> Result<(), NotifyError> {
        Err(self.0.clone())
    }
}

#[cfg(windows)]
fn icon_file(svc: &Services) -> Option<std::path::PathBuf> {
    let file = svc.paths.home.join("argus.ico");
    let same = std::fs::read(&file).is_ok_and(|have| have == argus_native::ARGUS_ICO);
    (same || std::fs::write(&file, argus_native::ARGUS_ICO).is_ok()).then_some(file)
}

#[cfg(windows)]
fn system(svc: &Services) -> Arc<dyn Notifier> {
    let identity = argus_native::Identity::argus(icon_file(svc));
    match argus_native::WindowsNotifier::new(&identity) {
        Ok(notifier) => Arc::new(notifier),
        Err(error) => Arc::new(Refusing(error)),
    }
}

#[cfg(not(windows))]
fn system(_svc: &Services) -> Arc<dyn Notifier> {
    let why = "OS toasts are only built for Windows";
    Arc::new(Refusing(NotifyError::Platform(why.to_owned())))
}

pub(crate) fn build_notifier(svc: &Services, opts: &ServeOptions) -> Option<Arc<dyn Notifier>> {
    let notify = &opts.notify;
    if !notify.enabled {
        return None;
    }
    let real = notify.system && !svc.shadow;
    Some(match &notify.notifier {
        Some(given) => Arc::clone(given),
        None if real => system(svc),
        None => Arc::new(RecordingNotifier::new()),
    })
}

/// The balloon that backs up a toast: the given one, else the tray's when the toast is the
/// system's.
fn fallback_of(svc: &Services, opts: &NotifyOptions) -> Option<Arc<dyn TrayFallback>> {
    let own = opts.notifier.is_none() && opts.system && !svc.shadow;
    opts.fallback
        .clone()
        .or_else(|| own.then(|| Arc::new(TrayBalloon::new(svc.tray.clone())) as _))
}

fn ports(svc: &Services, opts: &NotifyOptions, notifier: Arc<dyn Notifier>) -> Ports {
    Ports {
        store: Arc::new(NotifyDocs::new(Arc::clone(&svc.db))),
        config: Arc::new(LiveConfig::new(
            Arc::clone(&svc.config),
            svc.paths.clone(),
            svc.env_port,
        )),
        clock: opts.clock.clone().unwrap_or_else(|| Arc::new(SystemClock)),
        names: Arc::new(BoardNames::new(svc.boards())),
        activity: Arc::new(ActivityAdapter::of(svc)),
        events: Arc::new(DocStream),
        delivery: Delivery::new(notifier, fallback_of(svc, opts)),
    }
}

pub(crate) fn build(svc: &Services, opts: &ServeOptions) -> Option<Arc<Notify>> {
    let notifier = svc.notifier.clone()?;
    let port = svc.config().port;
    Some(Arc::new(Notify::new(
        ports(svc, &opts.notify, notifier),
        port,
    )))
}

type Export = (String, BoardFacts);

/// The pings of the exports, in order, off the exporting thread: a toast is a call to the
/// platform. The thread ends when the last sender is dropped.
fn worker(notify: Arc<Notify>) -> Option<Sender<Export>> {
    let (tx, rx) = mpsc::channel::<Export>();
    let run = move || {
        for (key, facts) in rx {
            drop(notify.on_board(&key, &facts));
        }
    };
    thread::Builder::new()
        .name("notify-board".to_owned())
        .spawn(run)
        .ok()
        .map(|_| tx)
}

fn announce(tx: &Sender<Export>, boards: &BoardReader, key: &str) {
    if let Some(board) = boards.board(key) {
        let facts = facts_of(key, &board, boards.board(MAIN_KEY).as_ref());
        drop(tx.send((key.to_owned(), facts)));
    }
}

pub(crate) fn start(svc: &Arc<Services>) {
    let Some(notify) = &svc.notify else {
        return;
    };
    notify.set_port(svc.port());
    notify.set_door(svc.door().cloned());
    notify.start();
    if let (Some(sync), Some(tx)) = (&svc.sync, worker(Arc::clone(notify))) {
        let boards = svc.boards();
        sync.on_board(move |key| announce(&tx, &boards, key));
    }
    let notify = Arc::clone(notify);
    svc.keep(svc.bus.on_reset(move |e| {
        if e.scope == PORTFOLIO {
            notify.mirror();
        }
    }));
}

pub(crate) fn stop(svc: &Services) {
    if let Some(notify) = &svc.notify {
        notify.stop();
    }
}
