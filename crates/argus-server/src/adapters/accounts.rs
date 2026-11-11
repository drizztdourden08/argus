//! The accounts service's docs: one `Accounts` doc per account and the state docs, all in
//! `_portfolio`.

use std::sync::Arc;

use argus_accounts::Account;
use argus_core::ids::PORTFOLIO;
use argus_core::{Coll, DocChange};
use argus_store::Db;
use serde_json::{Value, json, to_value};

/// Writes the accounts' docs into the doc store.
pub struct AccountsDocs {
    db: Arc<Db>,
}

impl AccountsDocs {
    /// The docs of a database.
    #[must_use]
    pub fn new(db: Arc<Db>) -> Self {
        AccountsDocs { db }
    }

    /// Sets one doc of `coll`.
    ///
    /// # Errors
    /// When the store refuses the write.
    pub fn put(&self, coll: Coll, id: &str, data: Value) -> Result<(), String> {
        self.apply(vec![DocChange::set(coll, id.to_owned(), data)])
    }

    /// Sets the `Accounts` doc of every account, with no usage read yet.
    ///
    /// # Errors
    /// When the store refuses the write.
    pub fn put_accounts(&self, accounts: &[Account]) -> Result<(), String> {
        let changes = accounts
            .iter()
            .map(|a| DocChange::set(Coll::Accounts, a.key.clone(), account_doc(a)))
            .collect();
        self.apply(changes)
    }

    fn apply(&self, changes: Vec<DocChange>) -> Result<(), String> {
        self.db
            .tx(|tx| {
                tx.apply(PORTFOLIO, &changes)?;
                Ok(())
            })
            .map_err(|e| e.to_string())
    }
}

/// The doc of an account before any usage was read: empty limits, nothing known of the logins.
#[must_use]
pub fn account_doc(account: &Account) -> Value {
    json!({
        "key": account.key,
        "label": account.label,
        "color": account.color,
        "active": false,
        "plan": null,
        "cli": { "loggedIn": false, "expiresAt": null, "orgId": account.desktop_org },
        "desktopLogin": { "saved": false, "savedAt": null },
        "limits": [],
        "readAt": null,
        "error": null,
        "burnPerMin": null,
        "rules": to_value(&account.rules).unwrap_or(Value::Null),
        "chats": { "total": 0, "archived": 0, "folders": 0 },
    })
}
