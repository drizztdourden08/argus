//! The Argus HTTP server: one listener on `127.0.0.1`, the Host, peer, Origin and content-type
//! checks of DESIGN 8, the db shim and action routes, the doc stream with frames written by hand
//! so the bytes match the Node server, the admin routes and the web build.
//!
//! Each service has one module in `services` (build, start, stop), one in `adapters` (its crate's
//! traits over the stores), one in `routes` (registered in [`routes::register_all`]) and one
//! test file; `docs/architecture.md` has the layout.
#![deny(clippy::cognitive_complexity)]

pub mod adapters;
pub mod boards;
mod body;
mod ctx;
pub mod endpoint;
pub mod first_start;
mod gate;
pub mod instance;
mod legacy_lead;
mod local_names;
mod localhost_door;
mod pathing;
pub mod preview;
mod reply;
mod router;
pub mod routes;
pub mod security;
mod serve_mode;
mod server;
mod services;
pub mod shadow;
pub mod sse;
mod web;

pub use ctx::{AuthKind, Ctx};
pub use local_names::{LiveNames, LocalNamesOptions};
pub use reply::{html_page, json_reply};
pub use router::{AuthMode, Handler, RouteMatch, RouteOpts, Router};
pub use serve_mode::ServeMode;
pub use server::{ServeError, ServeOptions, ServerHandle, serve};
pub use services::{
    AccountsOptions, AccountsService, AgentsOptions, CacheOptions, CacheService, ConsumerSource,
    ConsumerStatus, Consumers, EstimateState, LeadRef, LeadState, NodeStatus, NotifyOptions,
    ProcsOptions, Records, RecordsOptions, RunsOptions, Services, WakeHook, WakeOptions,
    WakeRequest, WorktreesOptions, estimate_state, learn_from_cycle,
};
pub use web::{PageVars, WebAssets, render_page};
