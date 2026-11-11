//! Accounts for the tests of this crate.

use crate::types::{Account, Rules};

/// An account with the given label and Desktop ids, default rules and no CLI folder.
pub(crate) fn account_in(
    key: &str,
    label: &str,
    desktop_account: Option<String>,
    desktop_org: Option<String>,
) -> Account {
    Account {
        key: key.to_owned(),
        label: label.to_owned(),
        color: "#000000".to_owned(),
        desktop_account,
        desktop_org,
        cli_dir: String::new(),
        email: None,
        plan: None,
        can_receive: true,
        rules: Rules::default(),
    }
}

/// An account whose label is its key and whose Desktop account is `uuid-<key>`.
pub(crate) fn account(key: &str) -> Account {
    account_in(key, key, Some(format!("uuid-{key}")), None)
}
