//! Which account a headless wake signs in with: the one that owns the session, unless it is at its limit
//! and another configured account has room. The usage is what Argus already holds for the tray.

use argus_accounts::Account;
use argus_core::Coll;
use argus_core::ids::PORTFOLIO;
use argus_store::Db;
use serde_json::Value;

/// The percent at which an account counts as spent.
const FULL: f64 = 100.0;
const FIVE_HOUR: &str = "five_hour";
const WEEKLY: &str = "weekly_all";

/// The use of one account, as far as it is known.
#[derive(Debug, Clone, PartialEq)]
pub struct Use {
    /// The account key.
    pub key: String,
    /// Percent of the 5-hour limit.
    pub five_hour: Option<f64>,
    /// Percent of the weekly limit.
    pub weekly: Option<f64>,
}

impl Use {
    fn full(&self) -> bool {
        self.load().is_some_and(|load| load >= FULL)
    }

    /// The highest known percent, or none when no limit is known.
    fn load(&self) -> Option<f64> {
        match (self.five_hour, self.weekly) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        }
    }
}

/// The account that signs the wake in, and why it is not the owner's when it is not.
#[derive(Debug, PartialEq)]
pub struct Chosen<'a> {
    /// The account.
    pub account: &'a Account,
    /// Set when the owning account was passed over.
    pub why: Option<String>,
}

fn percent(doc: &Value, kind: &str) -> Option<f64> {
    doc["limits"]
        .as_array()?
        .iter()
        .find(|limit| limit["kind"] == kind)?["percent"]
        .as_f64()
}

/// The uses the `Accounts` docs hold.
#[must_use]
pub fn uses_of(docs: &[Value]) -> Vec<Use> {
    docs.iter()
        .filter_map(|doc| {
            Some(Use {
                key: doc["key"].as_str()?.to_owned(),
                five_hour: percent(doc, FIVE_HOUR),
                weekly: percent(doc, WEEKLY),
            })
        })
        .collect()
}

/// The uses held in the database now; none when it cannot be read.
#[must_use]
pub fn uses_now(db: &Db) -> Vec<Use> {
    let rows = db
        .docs()
        .list(PORTFOLIO, Coll::Accounts)
        .unwrap_or_default();
    let docs: Vec<Value> = rows.into_iter().map(|row| row.data).collect();
    uses_of(&docs)
}

/// The account to sign in with: `owner` when it has room or its use is unknown, else the other account with
/// room and the lowest use, else `owner` again.
#[must_use]
pub fn pick<'a>(accounts: &'a [Account], owner: &'a Account, uses: &[Use]) -> Chosen<'a> {
    let of = |key: &str| uses.iter().find(|u| u.key == key);
    if !of(&owner.key).is_some_and(Use::full) {
        return Chosen {
            account: owner,
            why: None,
        };
    }
    let best = accounts
        .iter()
        .filter(|a| a.key != owner.key)
        .filter_map(|a| of(&a.key).filter(|u| !u.full()).map(|u| (a, u.load()?)))
        .min_by(|(_, x), (_, y)| x.total_cmp(y));
    match best {
        Some((account, _)) => Chosen {
            account,
            why: Some(format!("{} is at its limit", owner.key)),
        },
        None => Chosen {
            account: owner,
            why: None,
        },
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn account(key: &str) -> Account {
        let doc = json!({"key": key, "label": key, "color": "#000000", "cliDir": key});
        serde_json::from_value(doc).expect("account")
    }

    fn used(key: &str, five_hour: Option<f64>, weekly: Option<f64>) -> Use {
        Use {
            key: key.to_owned(),
            five_hour,
            weekly,
        }
    }

    fn key_of(accounts: &[Account], uses: &[Use]) -> (String, Option<String>) {
        let chosen = pick(accounts, &accounts[0], uses);
        (chosen.account.key.clone(), chosen.why)
    }

    #[test]
    fn the_owner_is_kept_when_it_has_room() {
        let accounts = [account("a"), account("b")];
        let uses = [
            used("a", Some(40.0), Some(99.0)),
            used("b", Some(0.0), None),
        ];
        assert_eq!(key_of(&accounts, &uses), ("a".to_owned(), None));
    }

    #[test]
    fn a_full_owner_gives_way_to_the_account_with_the_lowest_use() {
        let accounts = [account("a"), account("b"), account("c")];
        let uses = [
            used("a", Some(10.0), Some(100.0)),
            used("b", Some(70.0), Some(20.0)),
            used("c", Some(30.0), Some(10.0)),
        ];
        let (key, why) = key_of(&accounts, &uses);
        assert_eq!(key, "c");
        assert_eq!(why.as_deref(), Some("a is at its limit"));
    }

    #[test]
    fn the_five_hour_limit_counts_as_full_too() {
        let accounts = [account("a"), account("b")];
        let uses = [
            used("a", Some(100.0), Some(5.0)),
            used("b", None, Some(1.0)),
        ];
        assert_eq!(key_of(&accounts, &uses).0, "b");
    }

    #[test]
    fn nothing_known_or_nobody_with_room_keeps_the_owner() {
        let accounts = [account("a"), account("b")];
        assert_eq!(key_of(&accounts, &[]), ("a".to_owned(), None));
        let unknown_other = [used("a", None, Some(100.0))];
        assert_eq!(key_of(&accounts, &unknown_other).0, "a");
        let all_full = [used("a", None, Some(100.0)), used("b", Some(100.0), None)];
        assert_eq!(key_of(&accounts, &all_full), ("a".to_owned(), None));
    }

    #[test]
    fn the_uses_come_from_the_account_docs() {
        let docs = [
            json!({"key": "a", "limits": [
                {"kind": "five_hour", "percent": 5}, {"kind": "weekly_all", "percent": 42}]}),
            json!({"key": "b"}),
        ];
        let uses = uses_of(&docs);
        assert_eq!(uses[0], used("a", Some(5.0), Some(42.0)));
        assert_eq!(uses[1], used("b", None, None));
    }
}
