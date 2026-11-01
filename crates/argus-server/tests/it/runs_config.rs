//! The run service reads `config.runs` and `config.attach.maxBytes` on every call, as the Node
//! service does: a change to the live config applies to the next start and the next
//! attachment, with no restart.
//! K2: the live config limits of the run service.
// test-tag: standard

use crate::runs_rig::Rig;
use argus_testkit::{Request, TestOptions, png};
use serde_json::json;

fn set_limits(r: &Rig, max_concurrent: u64, attach_max: u64) {
    let mut config = r.t.svc.config.write().expect("config lock");
    config.runs.max_concurrent = max_concurrent;
    config.attach.max_bytes = attach_max;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_changed_concurrency_limit_applies_to_the_next_start() {
    let r = Rig::with(TestOptions::default()).await;
    r.define(&[("a", json!({})), ("b", json!({})), ("c", json!({}))]);
    set_limits(&r, 1, 10 << 20);
    assert_eq!(r.begin("a").await["status"], "running");
    assert_eq!(r.begin("b").await["status"], "queued", "the limit is one");
    set_limits(&r, 3, 10 << 20);
    assert_eq!(r.begin("c").await["status"], "running", "the new limit");
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_changed_attachment_limit_applies_to_the_next_attachment() {
    let r = Rig::with(TestOptions::default()).await;
    r.define(&[("job", json!({}))]);
    let run = r.begin("job").await;
    let pid = u32::try_from(run["pid"].as_u64().expect("a pid")).expect("a pid fits");
    r.host.exit(pid, Some(0)).await;
    r.until_status("R-1", "exited").await;
    let attach = || {
        let request = Request::new("POST", "/api/p/alpha/runs/R-1/attachments")
            .header("Content-Type", "image/png")
            .body(png());
        r.t.send(r.t.page(request))
    };
    set_limits(&r, 4, 10);
    let small = attach().await;
    assert_eq!(small.status, 413, "{}", small.text());
    set_limits(&r, 4, 1 << 20);
    let roomy = attach().await;
    assert_eq!(roomy.status, 200, "{}", roomy.text());
    r.t.stop(false).await;
}
