//! The toasts about account usage. The usage poller hands every poll to [`AccountAlerts::on_poll`];
//! the pure rules of `argus_accounts::alerts` say what to tell, and each alert goes out through
//! the notifications service, so it is shown and recorded like the other automatic pings.

use std::sync::{Mutex, PoisonError};

use argus_accounts::alerts::{
    Alert, AlertKind, AlertSettings, Alerter, Snapshot, local_offset_seconds,
};
use argus_core::clock::now_ms;
use argus_notify::{By, Level, PingRequest};

use crate::Services;

/// The last poll and what has been told about it.
#[derive(Default)]
pub struct AccountAlerts {
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    previous: Vec<Snapshot>,
    alerter: Alerter,
}

impl AccountAlerts {
    /// Takes the result of one poll: one snapshot per account.
    pub fn on_poll(&self, svc: &Services, snapshots: Vec<Snapshot>) {
        let Some(accounts) = &svc.accounts else {
            return;
        };
        let config = accounts.config();
        let settings = AlertSettings {
            warn_at: config.warn_at,
            propose_at: config.propose_at,
            port: svc.port(),
            offset_seconds: local_offset_seconds(),
        };
        let alerts = {
            let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            let State { previous, alerter } = &mut *state;
            let alerts = alerter.check(previous, &snapshots, &settings, now_ms());
            *previous = snapshots;
            alerts
        };
        if let Some(notify) = &svc.notify {
            for alert in alerts {
                notify.send(request_of(alert));
            }
        }
    }
}

fn level_of(kind: AlertKind) -> Level {
    match kind {
        AlertKind::ProposeSwitch => Level::Urgent,
        AlertKind::Warn | AlertKind::Reset => Level::Info,
    }
}

fn request_of(alert: Alert) -> PingRequest {
    PingRequest {
        title: Some(alert.title),
        text: alert.text,
        level: level_of(alert.kind),
        project: None,
        thread: None,
        by: By::Auto,
        url: Some(alert.url),
    }
}

#[cfg(test)]
mod tests {
    use argus_accounts::alerts::{Alert, AlertKind};
    use argus_notify::{By, Level};

    use super::request_of;

    fn alert(kind: AlertKind) -> Alert {
        Alert {
            kind,
            account: "second".to_owned(),
            target: None,
            title: "Second hit its weekly limit".to_owned(),
            text: "100% used.".to_owned(),
            url: "http://127.0.0.1:4000/?tab=accounts".to_owned(),
        }
    }

    #[test]
    fn a_proposal_is_urgent_and_the_rest_are_info() {
        assert_eq!(
            request_of(alert(AlertKind::ProposeSwitch)).level,
            Level::Urgent
        );
        assert_eq!(request_of(alert(AlertKind::Warn)).level, Level::Info);
        assert_eq!(request_of(alert(AlertKind::Reset)).level, Level::Info);
    }

    #[test]
    fn an_alert_pings_as_argus_with_its_address() {
        let ping = request_of(alert(AlertKind::Warn));
        assert_eq!(ping.by, By::Auto);
        assert_eq!(
            ping.url.as_deref(),
            Some("http://127.0.0.1:4000/?tab=accounts")
        );
        assert_eq!(ping.title.as_deref(), Some("Second hit its weekly limit"));
    }
}
