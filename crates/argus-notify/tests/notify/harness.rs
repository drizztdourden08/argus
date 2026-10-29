//! The service wired to fakes, and a reader of the toasts the fake notifier was given.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, PoisonError};

use argus_native::{Notifier, NotifyError, Payload, RecordingNotifier, Toast};
use argus_notify::fakes::{
    MemoryConfig, MemoryStore, RecordingActivity, RecordingEvents, StaticNames,
};
use argus_notify::{
    Delivery, ManualClock, Notify, NotifyConfig, Outcome, PingRequest, Ports, TrayFallback,
};

pub const T0: i64 = 1_793_016_000_000;
pub const PORT: u16 = 47615;
pub const TAB: &str = "http://127.0.0.1:47615/#notifications";
pub const ROTP: &str = "http://127.0.0.1:47615/p/rotp/";

pub fn from_lead(text: &str) -> PingRequest {
    PingRequest {
        project: Some("rotp".into()),
        thread: Some("lead".into()),
        ..PingRequest::new(text)
    }
}

#[derive(Default)]
pub struct ScriptedNotifier {
    inner: RecordingNotifier,
    script: Mutex<VecDeque<Result<(), NotifyError>>>,
}

impl ScriptedNotifier {
    pub fn script(&self, results: Vec<Result<(), NotifyError>>) {
        *self.script.lock().unwrap_or_else(PoisonError::into_inner) = results.into();
    }

    pub fn shown(&self) -> Vec<Payload> {
        self.inner.shown()
    }
}

impl Notifier for ScriptedNotifier {
    fn show(&self, toast: &Toast) -> Result<(), NotifyError> {
        let next = self
            .script
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .pop_front();
        match next {
            Some(Err(error)) => Err(error),
            _ => self.inner.show(toast),
        }
    }
}

#[derive(Default)]
pub struct ScriptedTray {
    answers: Mutex<VecDeque<bool>>,
    asked: Mutex<Vec<(String, String)>>,
}

impl ScriptedTray {
    pub fn answer(&self, answers: Vec<bool>) {
        *self.answers.lock().unwrap_or_else(PoisonError::into_inner) = answers.into();
    }

    pub fn asked(&self) -> Vec<(String, String)> {
        self.asked
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl TrayFallback for ScriptedTray {
    fn balloon(&self, title: &str, text: &str) -> bool {
        self.asked
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((title.to_owned(), text.to_owned()));
        self.answers
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .pop_front()
            .unwrap_or(false)
    }
}

#[derive(Debug)]
pub struct Shown {
    pub texts: Vec<String>,
    pub url: Option<String>,
    pub urgent: bool,
    pub silent: bool,
    pub tag: Option<String>,
}

impl Shown {
    fn parse(payload: &Payload) -> Self {
        let xml = &payload.xml;
        Self {
            texts: xml
                .split("<text>")
                .skip(1)
                .map(|part| part.split("</text>").next().unwrap_or("").to_owned())
                .collect(),
            url: xml
                .split_once("launch=\"")
                .map(|(_, rest)| rest.split('"').next().unwrap_or("").to_owned()),
            urgent: xml.contains("scenario=\"reminder\""),
            silent: xml.contains("<audio silent=\"true\"/>"),
            tag: payload.tag.clone(),
        }
    }

    pub fn is_summary(&self) -> bool {
        self.texts.first().is_some_and(|title| title == "Argus")
    }
}

pub struct Harness {
    pub notify: Notify,
    pub store: Arc<MemoryStore>,
    pub config: Arc<MemoryConfig>,
    pub clock: Arc<ManualClock>,
    pub notifier: Arc<ScriptedNotifier>,
    pub tray: Arc<ScriptedTray>,
    pub activity: Arc<RecordingActivity>,
    pub events: Arc<RecordingEvents>,
}

impl Harness {
    pub fn new() -> Self {
        Self::with(NotifyConfig::default())
    }

    pub fn with(config: NotifyConfig) -> Self {
        Self::build(
            config,
            StaticNames::default(),
            Arc::default(),
            Arc::new(ManualClock::at(T0)),
        )
    }

    pub fn named(config: NotifyConfig) -> Self {
        let names = StaticNames::of(&[("rotp", "Rotp")]);
        Self::build(config, names, Arc::default(), Arc::new(ManualClock::at(T0)))
    }

    pub fn restart(&self) -> Self {
        let names = StaticNames::of(&[("rotp", "Rotp")]);
        Self::build(
            self.config.get(),
            names,
            self.store.clone(),
            self.clock.clone(),
        )
    }

    fn build(
        config: NotifyConfig,
        names: StaticNames,
        store: Arc<MemoryStore>,
        clock: Arc<ManualClock>,
    ) -> Self {
        let config = Arc::new(MemoryConfig::new(config));
        let notifier = Arc::new(ScriptedNotifier::default());
        let tray = Arc::new(ScriptedTray::default());
        let activity = Arc::new(RecordingActivity::default());
        let events = Arc::new(RecordingEvents::default());
        let delivery = Delivery::new(notifier.clone(), Some(tray.clone()));
        let ports = Ports {
            store: store.clone(),
            config: config.clone(),
            clock: clock.clone(),
            names: Arc::new(names),
            activity: activity.clone(),
            events: events.clone(),
            delivery,
        };
        Self {
            notify: Notify::new(ports, PORT),
            store,
            config,
            clock,
            notifier,
            tray,
            activity,
            events,
        }
    }

    pub fn minute(&self) {
        self.clock.advance(61_000);
    }

    pub fn ping(&self, text: &str) -> Outcome {
        self.notify.send(PingRequest::new(text))
    }

    pub fn shown(&self) -> Vec<Shown> {
        self.notifier.shown().iter().map(Shown::parse).collect()
    }
}
