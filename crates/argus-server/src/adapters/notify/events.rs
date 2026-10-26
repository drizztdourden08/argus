//! What the page is told when a notification is recorded.

use argus_notify::{Notification, PageEvents};

/// The page hears of a notification through its doc: the store writes `n-<n>` in the same
/// transaction as the row, the bus carries the change and the fan-out sends it to every open
/// doc stream. Nothing else is pushed.
pub struct DocStream;

impl PageEvents for DocStream {
    fn recorded(&self, _notification: &Notification) {}
}
