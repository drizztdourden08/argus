//! The wake relay: plugs the server's port into the wake service once every service is built, because the
//! wake service is built before the accounts and the notifications it reaches. Without the accounts service
//! (a test server) the relay stays off, so no test starts a real CLI. See `docs/wake-relay.md`.

use std::sync::Arc;

use argus_accounts::{CliCredentials, Credentials, find_app_data};
use argus_wake::service::driver::lock;

use crate::Services;
use crate::adapters::wake::account_pick::uses_now;
use crate::adapters::wake::backoff::Backoff;
use crate::adapters::wake::relay_archived::ArchivedGate;
use crate::adapters::wake::relay_outcome::SignInGate;
use crate::adapters::wake::{RelayHost, ServerRelay};

pub(crate) fn wire(svc: &Services) {
    let (Some(wake), Some(accounts)) = (&svc.wake, svc.accounts.clone()) else {
        return;
    };
    let app_of = Arc::clone(&accounts);
    let list_of = Arc::clone(&accounts);
    let refused_of = Arc::clone(&accounts);
    let db = Arc::clone(&svc.db);
    let creds: Arc<dyn Credentials> = Arc::new(CliCredentials::new(find_app_data(
        accounts.config().app_data.as_deref(),
    )));
    let host = RelayHost {
        coord_home: svc.paths.coord_home.clone(),
        shadow: svc.shadow,
        app_data: Box::new(move || find_app_data(app_of.config().app_data.as_deref())),
        notify: svc.notify.clone(),
        hook: svc.wake_hook(),
        activity: Some(Arc::clone(&svc.activity)),
        sign_in: SignInGate::default(),
        archived: ArchivedGate::default(),
        accounts: Box::new(move || list_of.config().accounts),
        creds,
        usage: Box::new(move || uses_now(&db)),
        backoff: Backoff::default(),
        refused: Box::new(move |key| refused_of.record_refusal(key)),
    };
    lock(wake).set_relay_port(Arc::new(ServerRelay::new(host)));
}
