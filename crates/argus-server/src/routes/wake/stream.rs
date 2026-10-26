//! The wake stream: a listener attached to the engine, its events written as `wake` frames.

use std::sync::{Arc, Mutex, PoisonError};

use argus_core::ApiError;
use argus_wake::engine::{AttachOptions, ListenerId, ListenerMessage, channel};
use argus_wake::external::{clean_label, listener_cap_message};
use argus_wake::service::driver::{SharedWake, lock};
use serde_json::{Value, json, to_value};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_util::sync::CancellationToken;

use super::{service, thread};
use crate::Ctx;
use crate::router::Reply;
use crate::routes::params::key;
use crate::sse::{SseSender, open_stream};

type Slot = Arc<Mutex<Option<ListenerId>>>;

/// Writes what the engine sends until the stream is closed or the server stops. When it
/// returns, the sender drops and the stream ends after what is queued.
async fn forward(
    mut messages: UnboundedReceiver<ListenerMessage>,
    sender: SseSender,
    stop: CancellationToken,
) {
    loop {
        let message = tokio::select! {
            m = messages.recv() => m,
            () = stop.cancelled() => None,
        };
        let Some(message) = message else {
            return;
        };
        match message {
            ListenerMessage::Wake(event) => {
                let data = to_value(&event).unwrap_or(Value::Null);
                if !sender.event_with("wake", &data, &event.id) {
                    return;
                }
            }
            ListenerMessage::Replaced => {
                let _ = sender.event("replaced", &json!({ "reason": "newer listener" }), None);
                return;
            }
        }
    }
}

/// Detaches the listener when the connection ends.
fn leave(wake: &SharedWake, slot: &Slot) -> Box<dyn FnOnce() + Send> {
    let (wake, slot) = (Arc::clone(wake), Arc::clone(slot));
    Box::new(move || {
        let id = slot.lock().unwrap_or_else(PoisonError::into_inner).take();
        if let Some(id) = id {
            lock(&wake).detach(&id);
        }
    })
}

pub(super) async fn stream(ctx: Ctx) -> Reply {
    let wake = service(&ctx)?;
    let scope = key(&ctx)?;
    let thread = thread(&ctx)?;
    let v = if ctx.query("v") == Some("2") { 2 } else { 1 };
    let label = ctx.query("id").and_then(clean_label);
    let mut service = lock(&wake);
    if service.engine().listener_cap_reached(&scope, &thread, v) {
        return Err(ApiError::unavailable(listener_cap_message(&scope)));
    }
    let slot = Slot::default();
    let (sender, response) = open_stream(
        ctx.svc.heartbeat,
        &ctx.svc.streams,
        Some(leave(&wake, &slot)),
    );
    let ready =
        json!({ "scope": scope, "thread": thread, "windowMs": service.engine().window_ms() });
    let _ = sender.event("ready", &ready, None);
    let (sink, messages) = channel();
    let attached = service.attach(&scope, &thread, Box::new(sink), AttachOptions { v, label });
    *slot.lock().unwrap_or_else(PoisonError::into_inner) = Some(attached.id);
    drop(service);
    tokio::spawn(forward(messages, sender, ctx.svc.stop.clone()));
    Ok(response)
}
