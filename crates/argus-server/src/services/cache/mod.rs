//! The build cache of Brock and Tessera. Brock's `docs/build-cache.md` is the contract: they only add entries, Argus
//! owns usage tracking and pruning. The service reads the cache, works out who uses each entry from the lockfiles of
//! the known projects and worktrees, and prunes once after the start, then daily, and on demand. Shadow mode reports
//! and prunes nothing.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use argus_cache::{BuildCache, Outcome, PruneError, Report, default_root};
use argus_core::ApiError;
use argus_core::ErrorCode;
use argus_core::clock::now_ms;
use argus_store::{ActivityLevel, NewActivity};

use crate::{ServeOptions, Services};

mod config;
mod places;
mod timer;

const KIND: &str = "build-cache";

/// How the build cache starts; the fields after `enabled` are seams for tests.
#[derive(Clone)]
pub struct CacheOptions {
    /// Run the service.
    pub enabled: bool,
    /// Prune on a timer: once after `buildCache.startDelayMs`, then daily.
    pub timers: bool,
    /// Another cache folder than `BROCK_BUILD_CACHE_DIR` or `~/.brock/cache/build-cache`.
    pub root: Option<PathBuf>,
}

impl Default for CacheOptions {
    fn default() -> Self {
        CacheOptions {
            enabled: true,
            timers: true,
            root: None,
        }
    }
}

/// The cache of a root, and the last prune.
pub struct CacheService {
    cache: BuildCache,
    last: Mutex<Option<Outcome>>,
}

impl CacheService {
    fn new(root: &std::path::Path) -> Self {
        CacheService {
            cache: BuildCache::new(root),
            last: Mutex::new(None),
        }
    }

    /// The last prune of this run.
    #[must_use]
    pub fn last_prune(&self) -> Option<Outcome> {
        self.last
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn remember(&self, outcome: &Outcome) {
        *self.last.lock().unwrap_or_else(PoisonError::into_inner) = Some(outcome.clone());
    }
}

pub(crate) fn build(_svc: &Services, opts: &ServeOptions) -> Option<Arc<CacheService>> {
    let options = &opts.cache;
    let root = options.root.clone().or_else(default_root)?;
    options.enabled.then(|| Arc::new(CacheService::new(&root)))
}

pub(crate) fn start(svc: &Arc<Services>) {
    if svc.cache.is_some() && svc.tasks.cache_timer && !svc.shadow {
        timer::start(svc);
    }
}

pub(crate) fn stop(svc: &Services) {
    timer::stop(svc);
}

fn service(svc: &Services) -> Result<&Arc<CacheService>, ApiError> {
    svc.cache
        .as_ref()
        .ok_or_else(|| ApiError::unavailable("the build cache service is not running"))
}

/// The rows and totals now. Blocks: the disk.
pub(crate) fn report(svc: &Services) -> Result<Report, ApiError> {
    let cache = service(svc)?;
    let places = places::known(svc);
    Ok(cache
        .cache
        .report(&places, config::grace_days(svc), now_ms()))
}

fn note(svc: &Services, level: ActivityLevel, text: String) {
    svc.activity.add(NewActivity::new(KIND, level, text));
}

fn skipped(svc: &Services, why: &str) -> ApiError {
    note(
        svc,
        ActivityLevel::Warn,
        format!("build cache prune skipped: {why}"),
    );
    ApiError::new(ErrorCode::Conflict, why.to_owned())
}

fn logged(svc: &Services, outcome: &Outcome, on_demand: bool) {
    if on_demand || !outcome.is_quiet() {
        let warn = !outcome.refused.is_empty() || !outcome.failed.is_empty();
        let level = if warn {
            ActivityLevel::Warn
        } else {
            ActivityLevel::Info
        };
        note(svc, level, outcome.summary());
    }
}

/// Prunes now. Blocks: the disk. Not in shadow mode, and not before every project's worktrees were scanned once, since
/// a worktree that is not known yet may use an entry.
pub(crate) fn prune(svc: &Services, on_demand: bool) -> Result<Outcome, ApiError> {
    let cache = service(svc)?;
    if svc.shadow {
        return Err(ApiError::new(
            ErrorCode::ReadOnly,
            "shadow mode: Argus prunes no build cache",
        ));
    }
    if !places::worktrees_known(svc) {
        return Err(skipped(svc, "the worktrees were not scanned yet"));
    }
    let places = places::known(svc);
    let outcome = cache
        .cache
        .prune(&places, config::grace_days(svc), now_ms())
        .map_err(|PruneError::Unreadable(why)| skipped(svc, &why))?;
    cache.remember(&outcome);
    logged(svc, &outcome, on_demand);
    Ok(outcome)
}
