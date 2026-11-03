//! Notifications to the owner (DESIGN 13.13), the port of `src/notify/index.mjs`.
//!
//! A thread, the Overlord or Argus pings the owner when he is needed. Every ping is recorded (the
//! newest 50) and, when its level says so, shown as a toast through `argus_native::Notifier`, with
//! the tray balloon as the fallback. `Notify` is the service; the server provides the traits of
//! `ports` (store, config, clock, names, activity, page events) and calls `Notify::on_board` on
//! each board export. The HTTP routes live in the server crate.

mod auto;
mod board;
mod clock;
mod config;
mod configure;
mod deliver;
pub mod fakes;
mod gate;
mod kinds;
mod level;
mod limits;
mod lines;
mod links;
mod naming;
mod ports;
mod record;
mod request;
mod seen;
mod send;
mod sentence;
mod service;
mod settings;
mod watch;

pub use argus_core::StorageError;
pub use argus_core::clock::{Clock, ManualClock, SystemClock};
pub use board::{BoardFacts, Bundle, EntryFact, TestFact};
pub use clock::{iso, parse_ms};
pub use config::{AutoConfig, BoardTopics, NotifyConfig, OwnerTopics};
pub use configure::SettingsPatch;
pub use deliver::{Delivered, Delivery, ToastRequest, TrayFallback};
pub use kinds::{By, ToastState};
pub use level::{Level, PerLevel};
pub use limits::{GLOBAL_PER_MINUTE, SUMMARY_EVERY_MS, THREAD_PER_HOUR, THREAD_PER_MINUTE};
pub use ports::{
    ActivityLevel, ActivityLine, ActivitySink, ConfigStore, NotifyStore, PageEvents, Ports,
    ProjectNames,
};
pub use record::{Filter, KEEP_ROWS, Message, NewNotification, Notification};
pub use request::{Outcome, PingRequest};
pub use sentence::sentence;
pub use service::Notify;
pub use settings::{SOUND_DEFAULTS, Settings, settings_of};
