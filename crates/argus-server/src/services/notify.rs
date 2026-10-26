//! The notifications to the owner and the OS toast. Not wired yet: both builds answer `None`.
//! The wiring fills the functions (`svc.notifier` is built first, the service needs it), adds
//! `adapters/notify.rs` and `routes/notify.rs`.

use std::sync::Arc;

use argus_native::Notifier;
use argus_notify::Notify;

use crate::{ServeOptions, Services};

pub(crate) fn build_notifier(_svc: &Services, _opts: &ServeOptions) -> Option<Arc<dyn Notifier>> {
    None
}

pub(crate) fn build(_svc: &Services, _opts: &ServeOptions) -> Option<Arc<Notify>> {
    None
}

pub(crate) fn start(_svc: &Arc<Services>) {}

pub(crate) fn stop(_svc: &Services) {}
