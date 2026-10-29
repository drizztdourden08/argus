//! The dashboard client of the CLI: the server's port and token from its files, a blocking HTTP
//! client, the health check, the start of a server that is down, and the event stream reader.

pub mod autostart;
pub mod config;
pub mod health;
pub mod http;
pub mod launch;
pub mod sse;

pub use autostart::dashboard_client;
pub use config::{
    DEFAULT_PORT, DashboardConfig, board_port, dashboard_config, pid_record, server_port,
};
pub use health::healthy;
pub use http::{Client, ClientError, DOWN_MESSAGE};
pub use sse::{Event, SseReader};
