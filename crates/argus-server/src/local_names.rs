//! The local names in the server (`localNames` in `config.json`, off by default): the DNS
//! responder on `<address>:53` and the front door on `<address>:80`, both on the loopback address
//! of Argus, which nothing else on the machine binds. The door routes by Host exactly as the
//! main listener does. When port 80 cannot be bound (`http.sys` can own it on every address) the
//! door keeps the configured port instead and the names then carry that port. Nothing here
//! changes a system setting: the owner points `.<suffix>` at the responder with one NRPT rule.

use std::net::SocketAddr;
use std::sync::Arc;

use argus_core::names::{Door, LocalNames};
use argus_dns::{Responder, Zone};
use serde_json::{Value, json};
use tokio::net::TcpListener;

use crate::Services;

/// The ports to try, which tests move off 53 and 80.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalNamesOptions {
    /// The DNS port on the address (0 picks a free one).
    pub dns_port: u16,
    /// The port the front door tries first (0 picks a free one).
    pub door_port: u16,
}

impl Default for LocalNamesOptions {
    fn default() -> Self {
        Self {
            dns_port: 53,
            door_port: argus_core::names::HTTP_PORT,
        }
    }
}

/// The names that are up: the door and the DNS socket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveNames {
    /// The front door.
    pub door: Door,
    /// Where the responder listens.
    pub dns: SocketAddr,
    /// Whether the responder serves TCP too.
    pub dns_tcp: bool,
}

/// The sockets of the names, bound and not yet serving.
pub(crate) struct Front {
    pub live: LiveNames,
    pub listener: TcpListener,
    pub responder: Responder,
}

fn names_of(svc: &Services) -> Option<LocalNames> {
    match LocalNames::of(&svc.config()) {
        Ok(names) => Some(names).filter(|n| n.enabled),
        Err(why) => {
            svc.log.warn("local names refused", json!({ "why": why }));
            None
        }
    }
}

async fn bind_door(svc: &Services, names: &LocalNames, first: u16) -> Option<TcpListener> {
    let address = names.address;
    let first_try = TcpListener::bind((address, first)).await;
    let error = match first_try {
        Ok(listener) => return Some(listener),
        Err(error) => error,
    };
    let port = svc.port();
    svc.log.warn(
        &format!("local names: cannot bind {address}:{first}, the front door uses {address}:{port} and the names carry the port"),
        json!({ "err": error.to_string() }),
    );
    match TcpListener::bind((address, port)).await {
        Ok(listener) => Some(listener),
        Err(error) => {
            let line = format!("local names: cannot bind {address}:{port} either, names stay off");
            svc.log.warn(&line, json!({ "err": error.to_string() }));
            None
        }
    }
}

async fn bind_dns(svc: &Services, names: &LocalNames, port: u16) -> Option<Responder> {
    let zone = Zone::new(&names.suffix, names.address);
    let at = SocketAddr::from((names.address, port));
    match Responder::bind(at, zone).await {
        Ok(responder) => Some(responder),
        Err(error) => {
            let line = format!("local names: cannot bind DNS on {at}, names stay off");
            svc.log.warn(&line, json!({ "err": error.to_string() }));
            None
        }
    }
}

/// Binds the door and the responder when the names are on and valid; one warning in the log and
/// `None` when a socket cannot be had. The main listener must be bound first: its port is the
/// door's fallback.
pub(crate) async fn bind(svc: &Services, opts: &LocalNamesOptions) -> Option<Front> {
    let names = names_of(svc)?;
    let listener = bind_door(svc, &names, opts.door_port).await?;
    let responder = bind_dns(svc, &names, opts.dns_port).await?;
    let door = Door {
        address: names.address,
        suffix: names.suffix,
        port: listener.local_addr().ok()?.port(),
    };
    let live = LiveNames {
        door,
        dns: responder.local_addr().ok()?,
        dns_tcp: responder.has_tcp(),
    };
    Some(Front {
        live,
        listener,
        responder,
    })
}

/// Starts the responder; it ends with the server, and a failure of its socket is logged.
pub(crate) fn spawn_dns(svc: &Arc<Services>, responder: Responder) {
    let svc = Arc::clone(svc);
    tokio::spawn(async move {
        if let Err(error) = responder.run(svc.stop.clone()).await {
            svc.log.error(
                "local names: the DNS socket failed",
                json!({ "err": error.to_string() }),
            );
        }
    });
}

impl Services {
    /// The names that are up, when they are on and their sockets were bound.
    #[must_use]
    pub fn names(&self) -> Option<&LiveNames> {
        self.names.get()
    }

    /// The front door of the names, when they are up.
    #[must_use]
    pub fn door(&self) -> Option<&Door> {
        self.names().map(|live| &live.door)
    }
}

/// The `localNames` object of the health answer; `None` while the names are not up.
pub(crate) fn health_json(svc: &Services) -> Option<Value> {
    let live = svc.names()?;
    Some(json!({
        "address": live.door.address.to_string(),
        "suffix": live.door.suffix,
        "doorPort": live.door.port,
        "dns": live.dns.to_string(),
        "dnsTcp": live.dns_tcp,
    }))
}
