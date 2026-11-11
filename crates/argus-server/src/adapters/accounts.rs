//! The accounts service's docs: one `Accounts` doc per account and the state docs, all in
//! `_portfolio`.

use std::sync::Arc;

use argus_accounts::sync::{StepSink, SyncStep};
use argus_accounts::{Account, PollResult};
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
        self.apply(&[DocChange::set(coll, id.to_owned(), data)])
    }

    /// Sets the `Accounts` doc of every account, with no usage read yet.
    ///
    /// # Errors
    /// When the store refuses the write.
    pub fn put_accounts(&self, accounts: &[Account]) -> Result<(), String> {
        let changes = accounts
            .iter()
            .map(|a| DocChange::set(Coll::Accounts, a.key.clone(), account_doc(a)))
            .collect::<Vec<_>>();
        self.apply(&changes)
    }

    /// Writes what a poll found into the `Accounts` docs, keeping the fields other pieces own.
    /// `active` is the Desktop account uuid that is logged in now.
    ///
    /// # Errors
    /// When the store refuses the write.
    pub fn put_usage(
        &self,
        accounts: &[Account],
        results: &[PollResult],
        active: Option<&str>,
    ) -> Result<(), String> {
        let mut changes = Vec::new();
        for result in results {
            let Some(account) = accounts.iter().find(|a| a.key == result.key) else {
                continue;
            };
            let mut doc = self.current(account)?;
            let is_active = active.is_some() && active == account.desktop_account.as_deref();
            merge_usage(&mut doc, account, result, is_active);
            changes.push(DocChange::set(Coll::Accounts, account.key.clone(), doc));
        }
        self.apply(&changes)
    }

    fn current(&self, account: &Account) -> Result<Value, String> {
        let row = self
            .db
            .docs()
            .get(PORTFOLIO, Coll::Accounts, &account.key)
            .map_err(|e| e.to_string())?;
        Ok(row.map_or_else(|| account_doc(account), |r| r.data))
    }

    fn apply(&self, changes: &[DocChange]) -> Result<(), String> {
        self.db
            .tx(|tx| {
                tx.apply(PORTFOLIO, changes)?;
                Ok(())
            })
            .map_err(|e| e.to_string())
    }
}

impl StepSink for AccountsDocs {
    fn clear(&self) {
        let Ok(rows) = self.db.docs().list(PORTFOLIO, Coll::SyncSteps) else {
            return;
        };
        let gone = rows
            .into_iter()
            .map(|r| DocChange::delete(Coll::SyncSteps, r.id))
            .collect::<Vec<_>>();
        // A step log that cannot be cleared is shown stale; the run goes on.
        let _ = self.apply(&gone);
    }

    fn put(&self, step: &SyncStep) {
        let _ = self.put(Coll::SyncSteps, &step.doc_id(), step.to_doc());
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

fn merge_usage(doc: &mut Value, account: &Account, result: &PollResult, active: bool) {
    let Some(object) = doc.as_object_mut() else {
        return;
    };
    let org = account.desktop_org.clone();
    object.insert("active".to_owned(), json!(active));
    object.insert(
        "limits".to_owned(),
        to_value(&result.usage.limits).unwrap_or(Value::Null),
    );
    object.insert("readAt".to_owned(), json!(result.usage.read_at));
    object.insert("error".to_owned(), json!(result.usage.error));
    object.insert("burnPerMin".to_owned(), json!(result.burn_per_min));
    object.insert(
        "cli".to_owned(),
        json!({ "loggedIn": result.logged_in, "expiresAt": result.expires_at, "orgId": org }),
    );
}
