//! The process monitor's ports: its docs in the doc store, and its host (the projects and
//! their folders, the leads, the tray, the Overlord, the activity log).

use std::sync::{Arc, PoisonError, RwLock};

use argus_core::ids::PORTFOLIO;
use argus_core::{Coll, Config, DocChange};
use argus_native::{TrayFiles, read_live_tray};
use argus_procs::{
    Activity, Meta, ProcDoc, ProcsHost, ProcsStore, ProjectFolders, ProjectRoot, WakeRequest,
};
use argus_store::Db;
use serde_json::{Value, to_value};

use super::activity::ActivityAdapter;
use super::proc_folders::FolderReader;
use crate::boards::BoardReader;
use crate::services::{WakeHook, WakeRequest as Wake};

const KIND: &str = "procs";

/// Keeps the listing and its meta as docs of `_portfolio`.
pub struct ProcsDocs {
    db: Arc<Db>,
}

impl ProcsDocs {
    /// The docs of a database.
    #[must_use]
    pub fn new(db: Arc<Db>) -> Self {
        ProcsDocs { db }
    }
}

fn changes_of(docs: &[ProcDoc]) -> Result<Vec<DocChange>, String> {
    docs.iter()
        .map(|d| {
            let data = to_value(d).map_err(|e| e.to_string())?;
            Ok(DocChange::set(Coll::Procs, d.id.clone(), data))
        })
        .collect()
}

impl ProcsStore for ProcsDocs {
    fn replace(&self, docs: &[ProcDoc], meta: Option<&Meta>) -> Result<(), String> {
        let all = changes_of(docs)?;
        let meta: Option<Value> = meta.map(to_value).transpose().map_err(|e| e.to_string())?;
        self.db
            .tx(|tx| {
                tx.replace(PORTFOLIO, &[Coll::Procs], all)?;
                if let Some(data) = meta {
                    tx.apply(PORTFOLIO, &[DocChange::set(Coll::Meta, "procs", data)])?;
                }
                Ok(())
            })
            .map_err(|e| e.to_string())
    }
}

#[cfg(windows)]
pub(crate) fn alive(pid: u32) -> bool {
    argus_native::process_alive(pid)
}

#[cfg(not(windows))]
pub(crate) fn alive(pid: u32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

/// The dashboard around the monitor.
pub struct ProcsHostAdapter {
    boards: BoardReader,
    config: Arc<RwLock<Config>>,
    tray: TrayFiles,
    activity: ActivityAdapter,
    wake: WakeHook,
    folders: FolderReader,
}

impl ProcsHostAdapter {
    /// A host over the boards, the config, the tray's files, the activity log, the wake
    /// service and the reader of the project folders.
    #[must_use]
    pub fn new(
        boards: BoardReader,
        config: Arc<RwLock<Config>>,
        tray: TrayFiles,
        activity: ActivityAdapter,
        wake: WakeHook,
        folders: FolderReader,
    ) -> Self {
        ProcsHostAdapter {
            boards,
            config,
            tray,
            activity,
            wake,
            folders,
        }
    }
}

impl ProcsHost for ProcsHostAdapter {
    fn project_roots(&self) -> Vec<ProjectRoot> {
        let known = self
            .boards
            .projects()
            .into_iter()
            .filter_map(|p| p.root.map(|root| ProjectRoot { key: p.key, root }));
        let config = self.config.read().unwrap_or_else(PoisonError::into_inner);
        let set = config.project_roots.iter().map(|(key, root)| ProjectRoot {
            key: key.clone(),
            root: root.clone(),
        });
        known.chain(set).collect()
    }

    fn project_folders(&self) -> Vec<ProjectFolders> {
        self.folders.of(self.project_roots())
    }

    fn lead_of(&self, project: &str) -> Option<String> {
        self.boards.lead(project)
    }

    fn tray_pid(&self) -> Option<u32> {
        read_live_tray(&self.tray, alive).map(|r| r.pid)
    }

    fn overlord_board(&self) -> String {
        self.boards.overlord().unwrap_or_default()
    }

    fn lead(&self) -> Option<String> {
        self.boards.overlord_thread()
    }

    fn submit_wake(&self, wake: &WakeRequest) -> Result<(), String> {
        let request = Wake::about(
            &wake.scope,
            &wake.thread,
            &wake.reason,
            &wake.subject,
            &wake.text,
        );
        self.wake.submit(&[request])
    }

    fn activity(&self, entry: &Activity) {
        self.activity.procs_line(entry);
    }

    fn warn(&self, message: &str, detail: &str) {
        self.activity.warn(KIND, message, detail);
    }
}
