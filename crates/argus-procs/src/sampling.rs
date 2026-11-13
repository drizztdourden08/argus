//! One sample: take it, evaluate it, store it, and wake the Overlord about what is due.

use crate::alert_text::{ALERT_TAIL, alert_text};
use crate::alerts::{Due, Pace};
use crate::evaluate::{EvalInput, evaluate};
use crate::guard::GuardCtx;
use crate::matcher::match_rule;
use crate::meta::{MetaParts, build_meta, failed_meta};
use crate::num::to_millis;
use crate::ports::{Activity, Level, WakeRequest};
use crate::project::normalize_roots;
use crate::rule::Rule;
use crate::service::{Engine, Inner, RulesSnapshot, locked};
use crate::system_sampler::platform_name;
use crate::types::{Meta, Sample};
use std::time::Instant;

impl Inner {
    /// The guard of a sample: this process and its parents, the tray, the Windows folder.
    pub(crate) fn guard_for(&self, sample: &Sample) -> GuardCtx {
        let root = self
            .options
            .system_root
            .as_deref()
            .filter(|_| sample.is_windows());
        GuardCtx::new(
            &sample.procs,
            self.options.own_pid,
            self.ports.host.tray_pid(),
            root,
        )
    }

    /// A sample while holding the engine, so samples and kills never overlap.
    pub(crate) fn sample_serial(&self) -> Meta {
        let mut engine = locked(&self.engine);
        self.sample_once(&mut engine)
    }

    fn fill_responding(&self, sample: &mut Sample, rules: &RulesSnapshot) {
        let pids: Vec<u32> = sample
            .procs
            .iter()
            .filter(|p| p.responding.is_none())
            .filter(|p| match_rule(&rules.compiled, p).is_some_and(Rule::flags_hung))
            .map(|p| p.pid)
            .collect();
        if pids.is_empty() {
            return;
        }
        match self.ports.responder.responding(&pids) {
            Ok(answers) => {
                for p in &mut sample.procs {
                    p.responding = p.responding.or_else(|| answers.get(&p.pid).copied());
                }
            }
            Err(error) => self.ports.host.warn("process windows not checked", &error),
        }
    }

    fn meta_parts<'a>(
        &'a self,
        rules: &'a RulesSnapshot,
        at: i64,
        platform: &'a str,
    ) -> MetaParts<'a> {
        MetaParts {
            rules: &rules.rules,
            rules_file: &rules.file,
            rules_error: rules.error.as_deref(),
            sample_ms: self.options.config.sample_ms,
            sampled_at: at,
            platform,
        }
    }

    fn record_failure(&self, engine: &mut Engine, rules: &RulesSnapshot, error: &str) -> Meta {
        let message: String = error.chars().take(300).collect();
        let platform = platform_name();
        let previous = locked(&self.view).meta.clone();
        let meta = failed_meta(
            previous.as_ref(),
            &self.meta_parts(rules, 0, &platform),
            &message,
        );
        if engine.last_error.as_deref() != Some(message.as_str()) {
            self.ports.host.warn("process sample failed", &message);
        }
        engine.last_error = Some(message);
        locked(&self.view).meta = Some(meta.clone());
        self.publish_if_changed(None);
        meta
    }

    fn record_sample(
        &self,
        engine: &mut Engine,
        rules: &RulesSnapshot,
        sample: Sample,
        started: Instant,
    ) -> Meta {
        engine.last_error = None;
        let guard = self.guard_for(&sample);
        let roots = normalize_roots(&self.ports.host.project_roots());
        let input = EvalInput {
            sample: &sample,
            rules: &rules.rules,
            compiled: &rules.compiled,
            roots: &roots,
            guard: &guard,
        };
        let at = sample.at;
        let ev = evaluate(&mut engine.tracker, &input);
        let mut meta = build_meta(&ev, &self.meta_parts(rules, sample.at, &sample.platform));
        meta.took_ms = Some(to_millis(started.elapsed().as_secs_f64() * 1000.0));
        let pace = Pace {
            now: sample.at,
            repeat_ms: i64::try_from(self.options.config.repeat_ms).unwrap_or(i64::MAX),
        };
        let due = engine.alerts.due(&ev, &rules.rules, pace);
        self.reap(&mut engine.reaper, &sample, &guard);
        {
            let mut view = locked(&self.view);
            view.listing = ev.docs;
            view.raw = Some(sample);
            view.meta = Some(meta.clone());
        }
        self.publish_if_changed(Some(at));
        if !due.is_empty() {
            self.wake_overlord(&due);
        }
        meta
    }

    /// One sample, its docs and its alert. Never fails: a failed sample is recorded in the meta.
    pub(crate) fn sample_once(&self, engine: &mut Engine) -> Meta {
        let rules = self.rules_snapshot();
        let started = Instant::now();
        match self.ports.sampler.sample() {
            Err(error) => self.record_failure(engine, &rules, &error),
            Ok(mut sample) => {
                self.fill_responding(&mut sample, &rules);
                self.record_sample(engine, &rules, sample, started)
            }
        }
    }

    fn wake_overlord(&self, due: &[Due]) {
        let text = alert_text(due);
        let host = &self.ports.host;
        let wake = WakeRequest {
            scope: host.overlord_board(),
            thread: host
                .coordinator()
                .unwrap_or_else(|| "coordinator".to_string()),
            reason: "process-alert".to_string(),
            subject: "procs".to_string(),
            text: text.clone(),
        };
        if let Err(error) = host.submit_wake(&wake) {
            host.warn("process alert not sent", &error);
        }
        host.activity(&Activity {
            kind: "process-alert",
            level: Level::Warn,
            text: text.strip_suffix(ALERT_TAIL).unwrap_or(&text).to_string(),
        });
    }
}
