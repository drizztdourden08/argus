//! The wake relay: plugs the server's port into the wake service once every service is built, because the
//! wake service is built before the accounts and the notifications it reaches. Without the accounts service
//! (a test server) the relay stays off, so no test starts a real CLI. See `docs/wake-relay.md`.

use std::sync::Arc;

use argus_accounts::find_app_data;
use argus_wake::service::driver::lock;

use crate::Services;
use crate::adapters::wake::{RelayHost, ServerRelay};

pub(crate) fn wire(svc: &Services) {
    let (Some(wake), Some(accounts)) = (&svc.wake, svc.accounts.clone()) else {
        return;
    };
    let host = RelayHost {
        coord_home: svc.paths.coord_home.clone(),
        shadow: svc.shadow,
        app_data: Box::new(move || find_app_data(accounts.config().app_data.as_deref())),
        notify: svc.notify.clone(),
        hook: svc.wake_hook(),
    };
    lock(wake).set_relay_port(Arc::new(ServerRelay::new(host)));
}
