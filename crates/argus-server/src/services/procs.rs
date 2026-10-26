//! The process monitor: a [`ProcsService`] over the machine's processes, its docs in
//! `_portfolio`, the Overlord's rules file in the dashboard home.

use std::sync::Arc;

use argus_native::TrayFiles;
use argus_procs::{
    Killer, Options, Ports, PowerShellResponder, ProcsConfig, ProcsService, RULES_FILE, Responder,
    Sampler, SystemKiller, SystemSampler,
};
use serde_json::Value;

use crate::adapters::activity::ActivityAdapter;
use crate::adapters::procs::{ProcsDocs, ProcsHostAdapter};
use crate::{ServeOptions, Services};

/// How the process monitor starts; the fields after `enabled` are seams for tests.
#[derive(Clone)]
pub struct ProcsOptions {
    /// Run the monitor.
    pub enabled: bool,
    /// Sample on a timer.
    pub timer: bool,
    /// Another source of samples than the machine.
    pub sampler: Option<Arc<dyn Sampler>>,
    /// Another way to ask which windows hang than PowerShell.
    pub responder: Option<Arc<dyn Responder>>,
    /// Another way to end a process than the system's.
    pub killer: Option<Arc<dyn Killer>>,
}

impl Default for ProcsOptions {
    fn default() -> Self {
        ProcsOptions {
            enabled: true,
            timer: true,
            sampler: None,
            responder: None,
            killer: None,
        }
    }
}

fn system_root() -> Option<String> {
    cfg!(windows).then(|| std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into()))
}

fn ports(svc: &Services, opts: &ProcsOptions) -> Ports {
    let host = ProcsHostAdapter::new(
        svc.boards(),
        Arc::clone(&svc.config),
        TrayFiles::new(&svc.paths.home),
        ActivityAdapter::of(svc),
    );
    Ports {
        sampler: opts
            .sampler
            .clone()
            .unwrap_or_else(|| Arc::new(SystemSampler)),
        responder: opts
            .responder
            .clone()
            .unwrap_or_else(|| Arc::new(PowerShellResponder)),
        killer: opts
            .killer
            .clone()
            .unwrap_or_else(|| Arc::new(SystemKiller)),
        store: Arc::new(ProcsDocs::new(Arc::clone(&svc.db))),
        host: Arc::new(host),
    }
}

pub(crate) fn build(svc: &Services, opts: &ServeOptions) -> Option<Arc<ProcsService>> {
    let procs = &opts.procs;
    let setting = svc
        .config()
        .extra
        .get("procs")
        .cloned()
        .unwrap_or(Value::Null);
    let options = Options {
        config: ProcsConfig::from_json(&setting),
        rules_file: svc.paths.home.join(RULES_FILE),
        own_pid: std::process::id(),
        system_root: system_root(),
        timer: procs.timer,
    };
    procs
        .enabled
        .then(|| Arc::new(ProcsService::new(ports(svc, procs), options)))
}

pub(crate) fn start(svc: &Arc<Services>) {
    if let Some(procs) = &svc.procs {
        procs.start();
    }
}

pub(crate) fn stop(svc: &Services) {
    if let Some(procs) = &svc.procs {
        procs.stop();
    }
}
