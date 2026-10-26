//! Board presence: a board page opened with `?thread=<id>` holds this stream open for as long as
//! its tab is loaded, and the thread counts as having its board open.

use std::sync::{Arc, Mutex, PoisonError};

use argus_wake::external::{clean_tab, presence_thread};
use argus_wake::presence::PresenceToken;
use argus_wake::service::driver::lock;
use serde_json::json;

use super::{refusal, service};
use crate::Ctx;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::routes::params::key;
use crate::sse::{SseSender, open_stream};

/// What a presence stream keeps while it is open: its claim, and the sender that keeps the
/// connection from ending.
#[derive(Default)]
struct Hold {
    token: Option<PresenceToken>,
    sender: Option<SseSender>,
}

pub(super) async fn presence(ctx: Ctx) -> Reply {
    let wake = service(&ctx)?;
    let scope = key(&ctx)?;
    let thread = presence_thread(ctx.query("thread")).map_err(|e| refusal(&e))?;
    let tab = clean_tab(ctx.query("tab").unwrap_or(""));
    let slot: Arc<Mutex<Hold>> = Arc::default();
    let (service, kept) = (Arc::clone(&wake), Arc::clone(&slot));
    let leave = Box::new(move || {
        let held = std::mem::take(&mut *kept.lock().unwrap_or_else(PoisonError::into_inner));
        if let Some(token) = held.token {
            lock(&service).presence_detach(&token);
        }
    });
    let (sender, response) = open_stream(ctx.svc.heartbeat, &ctx.svc.streams, Some(leave));
    let _ = sender.event(
        "ready",
        &json!({ "scope": scope, "thread": thread, "tab": tab }),
        None,
    );
    let token = lock(&wake).presence_attach(&scope, &thread, &tab);
    *slot.lock().unwrap_or_else(PoisonError::into_inner) = Hold {
        token: Some(token),
        sender: Some(sender),
    };
    let stop = ctx.svc.stop.clone();
    tokio::spawn(async move {
        stop.cancelled().await;
        slot.lock()
            .unwrap_or_else(PoisonError::into_inner)
            .sender
            .take();
    });
    Ok(response)
}

pub(super) async fn open(ctx: Ctx) -> Reply {
    let wake = service(&ctx)?;
    let rows = lock(&wake).presence_list();
    Ok(json_reply(200, &json!({ "open": rows })))
}
