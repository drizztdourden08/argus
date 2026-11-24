//! The login of a headless wake of a lead, and the one activity line that says which account signed in.

use std::path::Path;

use argus_store::{ActivityLevel, NewActivity};
use argus_wake::relay::Ward;

use super::login::{Signed, login_of};
use super::relay::RelayHost;

impl RelayHost {
    /// The login for the headless wake of `target`. Without one the wake runs with the default CLI login and
    /// one activity line says why. A wake that signs in with another account than the owner's says so too.
    pub(super) fn headless_login(
        &self,
        app: &Path,
        session: &str,
        target: &Ward,
    ) -> Option<Signed> {
        let accounts = (self.accounts)();
        let uses = (self.usage)();
        match login_of(self.creds.as_ref(), &accounts, &uses, app, session) {
            Ok(signed) => {
                if let Some(why) = &signed.why {
                    self.log_wake(target, format!("signed in as {} ({why})", signed.account));
                }
                Some(signed)
            }
            Err(reason) => {
                self.log_wake(target, format!("default CLI login ({reason})"));
                None
            }
        }
    }

    /// One activity line about the headless wake of `target`.
    pub(super) fn log_wake(&self, target: &Ward, what: String) {
        let Some(activity) = &self.activity else {
            return;
        };
        let text = format!(
            "headless wake of {} {}: {what}",
            target.key.project, target.key.thread
        );
        activity.add(
            NewActivity::new("headless-wake", ActivityLevel::Info, text)
                .project(Some(target.key.project.clone()))
                .thread(Some(target.key.thread.clone())),
        );
    }
}
