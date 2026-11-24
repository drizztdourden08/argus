//! The hosted Overlord's server-side services (I-159).

mod attachment;
pub(crate) mod boot_resume;
pub(crate) mod boot_rule;
pub(crate) mod budget;
pub(crate) mod config;
pub(crate) mod control;
pub(crate) mod keeper;
pub(crate) mod live;
mod live_session;
mod live_transcript;
pub(crate) mod rules;
pub(crate) mod rules_tick;
pub(crate) mod running;
mod seat_register;
mod setting;
pub(crate) mod settings;
pub(crate) mod settings_limits;
pub(crate) mod shadow;
pub(crate) mod switch;
mod switch_board;
