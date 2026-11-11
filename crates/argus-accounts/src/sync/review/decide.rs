//! The plan of a review: the automatic plan with the owner's choices and edits laid over it,
//! as catalog changes and folder changes the sync job writes. The rows are worked out again
//! from a fresh scan; a choice must be one the row offers now.

use std::collections::BTreeMap;

use serde::Deserialize;

use super::diff::{Home, Look, home_record};
use super::records::Records;
use super::rows::{build_review, homes};
use super::{Review, owner_of};
use crate::catalog::plan::{ChangeKind, Plan, PlanSummary};
use crate::sidebar::{FolderEditor, FolderStep, chat_folder};
use crate::sync::job::{Env, Planned, Scan, kind_of, plan};

/// What the owner sends: a choice per row id, and edits applied in order after the choices.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Request {
    /// Row id to choice id; rows left out keep their default.
    #[serde(default)]
    pub decisions: BTreeMap<String, String>,
    /// The owner's own changes.
    #[serde(default)]
    pub edits: Vec<Edit>,
}

/// One edit of the owner, on one account.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum Edit {
    /// Files a chat in a folder, or in none.
    MoveChat {
        account: String,
        session_id: String,
        folder: Option<String>,
    },
    /// Archives a chat.
    Archive { account: String, session_id: String },
    /// Takes a chat out of the archive.
    Unarchive { account: String, session_id: String },
    /// Deletes an archived chat as the app does.
    DeleteChat { account: String, session_id: String },
    /// Renames a folder.
    RenameFolder {
        account: String,
        folder: String,
        name: String,
    },
    /// Deletes a folder; its chats are no longer filed.
    DeleteFolder { account: String, folder: String },
    /// Adds a folder at the end.
    CreateFolder { account: String, name: String },
}

/// Plans a review of a scan. `new_id` gives the ids of new folders.
///
/// # Errors
/// When the automatic plan fails, a row or a choice is unknown, an edit names an unknown
/// account, chat or folder, a chat to delete is not archived, or a file cannot be read.
pub fn plan_review(
    env: &Env,
    scan: Scan,
    request: &Request,
    new_id: &mut dyn FnMut() -> String,
) -> Result<Planned, String> {
    let auto = plan(env, scan)?;
    let review = build_review(&auto, env.accounts);
    let (changes, folders) = {
        let mut d = Decider {
            look: Look {
                planned: &auto,
                records: Records::from_auto(&auto.scan, &auto.catalog),
                homes: homes(&auto, env.accounts),
            },
            env,
            folders: FolderEditor::new(auto.folders.keys.clone(), auto.scan.keys.clone()),
            new_id,
        };
        for (row, choice) in &request.decisions {
            d.decide(&review, row, choice)?;
        }
        for edit in &request.edits {
            d.edit(edit)?;
        }
        let changes = d.look.records.changes(env.catalog, env.clock.now_ms())?;
        (changes, d.folders.finish(&auto.folders.steps))
    };
    let kinds = changes.iter().map(|c| kind_of(&auto.scan, c)).collect();
    let count = |kind| changes.iter().filter(|c| c.kind == kind).count();
    let summary = PlanSummary {
        scopes: auto.scan.scopes.len(),
        copies: count(ChangeKind::CopySession),
        updates: count(ChangeKind::UpdateSession),
        archive_indexes: count(ChangeKind::ArchiveIndex),
        writes: changes.len(),
        ..auto.catalog.summary.clone()
    };
    let catalog = Plan {
        changes,
        summary,
        ..auto.catalog
    };
    Ok(Planned {
        scan: auto.scan,
        catalog,
        kinds,
        folders,
    })
}

struct Decider<'a, 'e> {
    look: Look<'a>,
    env: &'a Env<'e>,
    folders: FolderEditor,
    new_id: &'a mut dyn FnMut() -> String,
}

impl Decider<'_, '_> {
    fn scope_name(&self, index: usize) -> String {
        self.look.planned.scan.scopes[index].scope.name()
    }

    fn scopes(&self) -> std::ops::Range<usize> {
        0..self.look.planned.scan.scopes.len()
    }

    fn owner(&self, index: usize) -> Option<&str> {
        owner_of(self.env.accounts, &self.look.planned.scan, index)
    }

    fn decide(&mut self, review: &Review, id: &str, choice: &str) -> Result<(), String> {
        let row = review
            .rows
            .iter()
            .find(|r| r.id == id)
            .ok_or_else(|| format!("unknown row {id}"))?;
        let diff = row
            .diff
            .as_ref()
            .ok_or_else(|| format!("row {id} has nothing to decide"))?;
        if !diff.choices.iter().any(|c| c.id == choice) {
            return Err(format!("{choice} is not a choice of row {id}"));
        }
        if choice == diff.default {
            return Ok(());
        }
        let (verb, key) = choice.split_once(':').unwrap_or((choice, ""));
        match id.strip_prefix("folder:") {
            Some(name) => self.decide_folder(name, verb, key),
            None => self.decide_chat(id, verb, key),
        }
    }

    fn decide_chat(&mut self, id: &str, verb: &str, key: &str) -> Result<(), String> {
        match verb {
            "skip" => {
                self.look.records.skip(id);
                self.revert_where(id, &|_| true);
            }
            "copy_to" => {
                let others = |owner: Option<&str>| owner.is_some_and(|o| o != key);
                let keep: Vec<bool> = self.scopes().map(|i| !others(self.owner(i))).collect();
                self.look.records.keep_copies(id, &|i| keep[i]);
                self.revert_where(id, &|i| !keep[i]);
            }
            "take" => {
                let source = home_record(&self.look, key, id)
                    .cloned()
                    .ok_or_else(|| format!("{key} has no chat {id}"))?;
                self.look.records.take_history(id, &source);
            }
            "archive_all" | "unarchive_all" => {
                self.look.records.archive_all(id, verb == "archive_all");
            }
            "link_all" => self.look.records.link_all(id),
            "unlink_all" => self.look.records.unlink_all(id),
            "folder_of" => self.folder_of(id, key)?,
            _ => return Err(format!("{verb} cannot be applied to chat {id}")),
        }
        Ok(())
    }

    /// Puts the chat back in its scanned folder in the scopes `pick` accepts.
    fn revert_where(&mut self, id: &str, pick: &dyn Fn(usize) -> bool) {
        for i in self.scopes().filter(|&i| pick(i)) {
            let scope = self.scope_name(i);
            if self.folders.has_scope(&scope) {
                self.folders.revert(&scope, id);
            }
        }
    }

    /// Files the chat, in every scope that holds it, in the folder it has on account `key`.
    fn folder_of(&mut self, id: &str, key: &str) -> Result<(), String> {
        let home = self.home(key)?;
        let name = chat_folder(&self.look.planned.scan.keys, &home.scope, id);
        for i in self.scopes() {
            let scope = self.scope_name(i);
            let holds = self.look.records.wanted(i, id).is_some();
            let fits = name
                .as_deref()
                .is_none_or(|n| self.folders.has_folder(&scope, n));
            if holds && fits && self.folders.has_scope(&scope) {
                self.folders.place(&scope, id, name.as_deref())?;
            }
        }
        Ok(())
    }

    fn decide_folder(&mut self, name: &str, verb: &str, key: &str) -> Result<(), String> {
        let created: Vec<(String, String)> = self
            .look
            .planned
            .folders
            .steps
            .iter()
            .filter_map(|s| match s {
                FolderStep::CreateFolder {
                    scope,
                    name: n,
                    new_id,
                    ..
                } if n == name => Some((scope.clone(), new_id.clone())),
                _ => None,
            })
            .collect();
        let accounts = self.env.accounts;
        let owner_of_name = |scope: &str| {
            let uuid = scope.split_once('/').map_or(scope, |(a, _)| a);
            accounts
                .iter()
                .find(|a| a.desktop_account.as_deref() == Some(uuid))
                .map(|a| a.key.clone())
        };
        for (scope, folder) in &created {
            let keep = verb == "create_on" && owner_of_name(scope).is_none_or(|o| o == key);
            if !keep {
                self.folders.drop_created(scope, folder);
            }
        }
        match verb {
            "skip" | "create_on" => Ok(()),
            "delete_on" => {
                for i in self.scopes() {
                    let scope = self.scope_name(i);
                    if self.folders.has_folder(&scope, name) {
                        self.folders.delete(&scope, name)?;
                    }
                }
                Ok(())
            }
            _ => Err(format!("{verb} cannot be applied to folder {name}")),
        }
    }

    fn home(&self, key: &str) -> Result<&Home, String> {
        self.look
            .homes
            .iter()
            .find(|h| h.key == key)
            .ok_or_else(|| format!("unknown account {key}"))
    }

    /// The account's home scope first, then its cross scopes.
    fn account_scopes(&self, key: &str) -> Result<Vec<usize>, String> {
        let home = self.home(key)?.index;
        let uuid = &self.look.planned.scan.scopes[home].scope.account;
        let cross = self
            .scopes()
            .filter(|&i| i != home && &self.look.planned.scan.scopes[i].scope.account == uuid);
        Ok(std::iter::once(home).chain(cross).collect())
    }

    fn edit(&mut self, edit: &Edit) -> Result<(), String> {
        match edit {
            Edit::MoveChat {
                account,
                session_id,
                folder,
            } => self.move_chat(account, session_id, folder.as_deref()),
            Edit::Archive {
                account,
                session_id,
            } => self.archive(account, session_id, true),
            Edit::Unarchive {
                account,
                session_id,
            } => self.archive(account, session_id, false),
            Edit::DeleteChat {
                account,
                session_id,
            } => self.delete_chat(account, session_id),
            Edit::RenameFolder {
                account,
                folder,
                name,
            } => self.on_folder(account, folder, &mut |ed, scope| {
                ed.rename(scope, folder, name)
            }),
            Edit::DeleteFolder { account, folder } => {
                self.on_folder(account, folder, &mut |ed, scope| ed.delete(scope, folder))
            }
            Edit::CreateFolder { account, name } => self.create_folder(account, name),
        }
    }

    /// The account's scopes, checking that its home scope will hold the chat.
    fn chat_scopes(&self, account: &str, id: &str) -> Result<Vec<usize>, String> {
        let scopes = self.account_scopes(account)?;
        if self.look.records.wanted(scopes[0], id).is_none() {
            return Err(format!("unknown chat {id} on {account}"));
        }
        Ok(scopes)
    }

    fn move_chat(&mut self, account: &str, id: &str, folder: Option<&str>) -> Result<(), String> {
        let scopes = self.chat_scopes(account, id)?;
        let home = self.scope_name(scopes[0]);
        if !self.folders.has_scope(&home) {
            return Err(format!("{account} has no sidebar folders"));
        }
        if let Some(name) = folder.filter(|n| !self.folders.has_folder(&home, n)) {
            return Err(format!("unknown folder {name} on {account}"));
        }
        for i in scopes {
            let scope = self.scope_name(i);
            let fits = folder.is_none_or(|n| self.folders.has_folder(&scope, n));
            if fits && self.folders.has_scope(&scope) && self.look.records.wanted(i, id).is_some() {
                self.folders.place(&scope, id, folder)?;
            }
        }
        Ok(())
    }

    fn archive(&mut self, account: &str, id: &str, archived: bool) -> Result<(), String> {
        let scopes = self.chat_scopes(account, id)?;
        self.look.records.archive_in(&scopes, id, archived);
        Ok(())
    }

    fn delete_chat(&mut self, account: &str, id: &str) -> Result<(), String> {
        let scopes = self.account_scopes(account)?;
        if self.look.records.current(scopes[0], id).is_none() {
            return Err(format!("unknown chat {id} on {account}"));
        }
        if self.look.records.archived(scopes[0], id) != Some(true) {
            return Err(format!(
                "chat {id} on {account} is not archived; only archived chats can be deleted"
            ));
        }
        for i in scopes {
            self.look.records.delete(i, id);
            let scope = self.scope_name(i);
            if self.folders.has_scope(&scope) {
                self.folders.forget(&scope, id);
            }
        }
        Ok(())
    }

    /// Runs a folder edit in each of the account's scopes that has the folder.
    fn on_folder(
        &mut self,
        account: &str,
        folder: &str,
        each: &mut dyn FnMut(&mut FolderEditor, &str) -> Result<(), String>,
    ) -> Result<(), String> {
        let scopes = self.account_scopes(account)?;
        if !self.folders.has_folder(&self.scope_name(scopes[0]), folder) {
            return Err(format!("unknown folder {folder} on {account}"));
        }
        for i in scopes {
            let scope = self.scope_name(i);
            if self.folders.has_folder(&scope, folder) {
                each(&mut self.folders, &scope).map_err(|e| format!("{account}: {e}"))?;
            }
        }
        Ok(())
    }

    fn create_folder(&mut self, account: &str, name: &str) -> Result<(), String> {
        let scopes = self.account_scopes(account)?;
        let home = self.scope_name(scopes[0]);
        if !self.folders.has_scope(&home) {
            return Err(format!("{account} has no sidebar folders"));
        }
        if self.folders.has_folder(&home, name) {
            return Err(format!("a folder named {name} already exists on {account}"));
        }
        for i in scopes {
            let scope = self.scope_name(i);
            if self.folders.has_scope(&scope) && !self.folders.has_folder(&scope, name) {
                let id = (self.new_id)();
                self.folders.create(&scope, name, &id)?;
            }
        }
        Ok(())
    }
}
