//! The notification routes (`POST /api/notify`, `/api/notify/test`, `GET` and `POST
//! /api/notify/settings`, `GET /api/notify`). Not wired yet: the notify wiring fills `register`.

use crate::Router;

/// Registers the notification routes.
pub fn register(_r: &mut Router) {}
