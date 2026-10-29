//! A temporary cache root, temporary project folders and a fixed clock. Each test file uses a part of it.
#![allow(dead_code)]

use argus_cache::{BuildCache, Outcome, Place, PlaceKind, PruneError, Report};
use argus_core::clock::{iso_from_ms, parse_ms};
use filetime::{FileTime, set_file_mtime};
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub const DAY: i64 = 86_400_000;

pub struct Rig {
    dir: TempDir,
    now: i64,
    places: RefCell<Vec<Place>>,
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().expect("parent")).expect("folders");
    fs::write(path, text).expect("file");
}

fn touch(path: &Path, ms: i64) {
    set_file_mtime(path, FileTime::from_unix_time(ms / 1000, 0)).expect("mtime");
}

impl Rig {
    pub fn without_cache() -> Self {
        Rig {
            dir: tempfile::tempdir().expect("temp dir"),
            now: parse_ms("2026-10-29T12:00:00Z").expect("date"),
            places: RefCell::default(),
        }
    }

    pub fn new() -> Self {
        let rig = Rig::without_cache();
        fs::create_dir_all(rig.root_path()).expect("cache root");
        rig
    }

    pub fn root_path(&self) -> PathBuf {
        self.dir.path().join("cache")
    }

    /// A folder full of files beside the cache root, for the links of the delete safety tests.
    pub fn sibling(&self) -> PathBuf {
        let dir = self.dir.path().join("sibling");
        argus_core::fixture::links::fill_sibling(&dir);
        dir
    }

    /// Sets the modification time of a folder to `ago` ms before the rig's now.
    pub fn age(&self, path: &Path, ago: i64) {
        touch(path, self.now - ago);
    }

    pub fn iso_days_ago(&self, days: i64) -> String {
        iso_from_ms(self.now - days * DAY)
    }

    pub fn name_folder(&self, kind: &str, name: &str) -> PathBuf {
        self.root_path().join(kind).join(name)
    }

    pub fn version_folder(&self, kind: &str, name: &str, version: &str) -> PathBuf {
        self.name_folder(kind, name).join(version)
    }

    pub fn exists(&self, kind: &str, name: &str, version: &str, key: &str) -> bool {
        self.version_folder(kind, name, version).join(key).exists()
    }

    /// An entry with a manifest, `used_ago` ms since its last use.
    pub fn entry(
        &self,
        kind: &str,
        name: &str,
        version: &str,
        key: &str,
        size: u64,
        used_ago: i64,
    ) {
        let dir = self.version_folder(kind, name, version).join(key);
        let used = iso_from_ms(self.now - used_ago);
        let manifest = format!(
            "{{\"schema\":1,\"kind\":\"{kind}\",\"sizeBytes\":{size},\"builtAt\":\"{used}\",\"lastUsedAt\":\"{used}\"}}"
        );
        write(&dir.join("manifest.json"), &manifest);
        write(&dir.join("files/index.js"), "x");
    }

    /// An entry whose manifest is missing; its folder time is `used_ago` ms old.
    pub fn bare_entry(&self, kind: &str, name: &str, version: &str, key: &str, used_ago: i64) {
        let dir = self.version_folder(kind, name, version).join(key);
        write(&dir.join("files/index.js"), "abc");
        touch(&dir, self.now - used_ago);
    }

    pub fn tmp(&self, name: &str, age: i64) {
        let dir = self.root_path().join(".tmp").join(name);
        write(&dir.join("half.js"), "x");
        touch(&dir, self.now - age);
    }

    /// A project folder `<label>` holding `pnpm-lock.yaml`.
    pub fn project(&self, label: &str, lock: &str) -> PathBuf {
        let root = self.dir.path().join("projects").join(label);
        write(&root.join("pnpm-lock.yaml"), lock);
        self.places_add(label, PlaceKind::Project, &root);
        root
    }

    /// A worktree of `project`; its lockfile is the installed one under `node_modules/.pnpm` when `installed`.
    pub fn worktree(&self, project: &str, name: &str, lock: &str, installed: bool) -> PathBuf {
        let root = self.dir.path().join("worktrees").join(name);
        let file = if installed {
            root.join("node_modules/.pnpm/lock.yaml")
        } else {
            root.join("pnpm-lock.yaml")
        };
        write(&file, lock);
        self.places_add(&format!("{project} / {name}"), PlaceKind::Worktree, &root);
        root
    }

    fn places_add(&self, label: &str, kind: PlaceKind, root: &Path) {
        self.places.borrow_mut().push(Place {
            label: label.to_owned(),
            kind,
            root: root.to_string_lossy().replace('\\', "/"),
        });
    }

    fn cache(&self) -> BuildCache {
        BuildCache::new(&self.root_path())
    }

    pub fn report(&self, grace: u32) -> Report {
        self.cache().report(&self.places.borrow(), grace, self.now)
    }

    pub fn prune(&self, grace: u32) -> Result<Outcome, PruneError> {
        self.cache().prune(&self.places.borrow(), grace, self.now)
    }
}
