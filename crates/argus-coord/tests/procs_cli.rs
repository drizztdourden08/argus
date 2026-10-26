//! Ports the `coord proc` and `coord activity` cases of `procs.http.test.mjs` against an Argus
//! test server with a fake machine: list and rules for anyone, rule set and rm and kill only for
//! the Overlord, and the newest lines of the activity log.

mod common;
#[allow(dead_code)]
#[path = "../../argus-server/tests/machine/mod.rs"]
mod machine;
mod remote;

use std::fs;
use std::sync::Arc;

use argus_core::clock::now_ms;
use argus_server::ServeOptions;
use machine::{Machine, row};
use remote::{ALPHA_LEAD, HELPER, MAIN, Rig, ok, refused, seed_standard};
use serde_json::{Value, json};

const OVERLORD: &str = "only the Overlord (the coordinator thread of project portfolio) can ";

async fn rig() -> (Rig, Arc<Machine>) {
    let fake = Arc::new(Machine::with_self());
    let wired = Arc::clone(&fake);
    let tweak = Arc::new(move |opts: &mut ServeOptions| {
        let rules = json!({ "rules": [{ "name": "test-kids", "match": { "cmd": "procs-http-test" }, "killable": true, "maxMemMB": 1000 }] });
        fs::create_dir_all(&opts.paths.home).expect("dashboard home");
        fs::write(
            opts.paths.home.join("process-rules.json"),
            rules.to_string(),
        )
        .expect("rules file");
        machine::install(&wired, opts);
    });
    (
        Rig::start_with(json!({}), seed_standard, Some(tweak)).await,
        fake,
    )
}

fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn proc(rig: &Rig, words: &[&str], session: Option<&str>, folder: &str) -> common::Ran {
    let mut argv = vec!["proc"];
    argv.extend(words);
    rig.run_in(&argv, session, &rig.dir.path().join(folder))
}

#[tokio::test(flavor = "multi_thread")]
async fn list_and_rules_work_for_anyone() {
    let (rig, fake) = rig().await;
    let pid = fake.add(row(41_005, "e", 70, now_ms() - 60_000));
    rig.t.write("POST", "/api/procs/sample", &json!({})).await;
    let list = proc(&rig, &["list"], Some(ALPHA_LEAD), "checkout-alpha");
    let out = ok(&list);
    assert!(
        out.contains(" processes, ") && out.contains("sampled"),
        "{out}"
    );
    assert!(
        flat(out).contains(&format!("{pid} node.exe test-kids 70 MB")),
        "{out}"
    );
    assert!(out.contains("protected (dashboard)"), "{out}");
    let json: Value = serde_json::from_str(ok(&proc(
        &rig,
        &["list", "--json", "--rule", "test-kids"],
        None,
        ".",
    )))
    .expect("json");
    let procs = json["procs"].as_array().expect("procs");
    assert!(!procs.is_empty() && procs.iter().all(|p| p["rule"] == "test-kids"));
    let rules = flat(ok(&proc(&rig, &["rules"], None, ".")));
    assert!(
        rules.contains("test-kids cmd~procs-http-test mem >1000 MB yes"),
        "{rules}"
    );
    rig.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn rule_set_rm_and_kill_are_for_the_overlord_only() {
    let (rig, fake) = rig().await;
    let pid = fake.add(row(41_006, "e", 70, now_ms() - 60_000));
    rig.t.write("POST", "/api/procs/sample", &json!({})).await;
    let pid_text = pid.to_string();
    let refusals = [
        (
            proc(
                &rig,
                &[
                    "rule",
                    "set",
                    "py",
                    "--name-match",
                    "^python",
                    "--max-mem",
                    "800",
                ],
                Some(ALPHA_LEAD),
                "checkout-alpha",
            ),
            "change the process rules; this session belongs to project alpha",
        ),
        (
            proc(
                &rig,
                &["rule", "set", "py", "--name-match", "^python"],
                Some(HELPER),
                "checkout-portfolio",
            ),
            "change the process rules; thread helper is not the coordinator",
        ),
        (
            proc(
                &rig,
                &["kill", &pid_text, "--reason", "testing it"],
                Some(ALPHA_LEAD),
                "checkout-alpha",
            ),
            "kill a process; this session belongs to project alpha",
        ),
        (
            proc(&rig, &["kill-rule", "test-kids"], None, "."),
            "kill processes; this session belongs to no project",
        ),
    ];
    for (ran, text) in &refusals {
        refused(
            ran,
            1,
            &format!("{OVERLORD}{text}. Ask it with coord lead portfolio"),
        );
    }
    let main = |words: &[&str]| proc(&rig, words, Some(MAIN), "checkout-portfolio");
    let set = main(&[
        "rule",
        "set",
        "py",
        "--name-match",
        "^python",
        "--max-mem",
        "800",
        "--idle",
        "2h",
        "--orphan",
        "--killable",
        "--note",
        "python servers",
    ]);
    let want = r#"added rule py: {"name":"py","match":{"name":"^python"},"maxMemMB":800,"idleMin":120,"orphan":true,"killable":true,"note":"python servers"}"#;
    assert!(ok(&set).starts_with(want), "{}", set.out);
    let changed = main(&["rule", "set", "py", "--max-mem", "none", "--no-orphan"]);
    assert!(
        ok(&changed).starts_with("changed rule py: ") && changed.out.contains("\"orphan\":false"),
        "{}",
        changed.out
    );
    refused(
        &main(&["rule", "set", "py", "--idle", "soon"]),
        1,
        "--idle takes a duration",
    );
    refused(
        &main(&["rule", "set", "py", "--cmd-match", "("]),
        1,
        "not a valid regex",
    );
    assert!(ok(&main(&["rule", "rm", "py"])).starts_with("removed rule py."));
    refused(&main(&["rule", "rm", "py"]), 1, "no rule \"py\"");
    refused(&main(&["kill", &pid_text]), 1, "--reason is required");
    refused(
        &main(&["kill", "999999", "--reason", "not there"]),
        1,
        "not in the listing",
    );
    let own = std::process::id().to_string();
    refused(
        &main(&["kill", &own, "--reason", "the server"]),
        1,
        "protected (dashboard)",
    );
    assert!(fake.alive(pid));
    let killed = main(&["kill", &pid_text, "--reason", "a test child"]);
    assert!(
        ok(&killed).starts_with(&format!("killed node.exe {pid} (70 MB).")),
        "{}",
        killed.out
    );
    assert!(!fake.alive(pid));
    let rule = main(&["kill-rule", "test-kids", "--older-than", "6h"]);
    assert!(
        ok(&rule).starts_with("test-kids: killed 0, skipped "),
        "{}",
        rule.out
    );
    let acts: Value = rig.t.get("/api/activity?kind=process-kill").await.json();
    let texts: Vec<&str> = acts["activity"]
        .as_array()
        .expect("activity")
        .iter()
        .filter_map(|a| a["text"].as_str())
        .collect();
    let line = format!(
        "killed node {pid} (procs-http-test, 70 MB) from the CLI (coordinator): a test child"
    );
    assert!(texts.iter().any(|t| *t == line), "{texts:?}");
    rig.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_proc_verbs_check_their_arguments_and_help_prints_the_usage() {
    let (rig, _fake) = rig().await;
    let main = |words: &[&str]| proc(&rig, words, Some(MAIN), "checkout-portfolio");
    refused(
        &main(&["rule"]),
        1,
        "coord proc rule set <name> ... | coord proc rule rm <name>",
    );
    refused(
        &main(&["rule", "set"]),
        1,
        "name the rule: coord proc rule set <name>.",
    );
    refused(
        &main(&["rule", "set", "py"]),
        1,
        "say what to set: --name-match",
    );
    refused(
        &main(&["rule", "set", "py", "--max-mem", "lots"]),
        1,
        "--max-mem takes megabytes (a whole number) or none.",
    );
    refused(
        &main(&["kill"]),
        1,
        "name the process: coord proc kill <pid>",
    );
    refused(
        &main(&["kill-rule"]),
        1,
        "name the rule: coord proc kill-rule <rule>",
    );
    refused(
        &main(&["frobnicate"]),
        1,
        "unknown proc command \"frobnicate\".",
    );
    let help = main(&["help"]);
    assert!(
        ok(&help).starts_with("coord proc list [--rule r] [--json] | rules [--json]"),
        "{}",
        help.out
    );
    rig.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn activity_prints_the_newest_lines_with_json_and_kind() {
    let (rig, _fake) = rig().await;
    rig.t.write("POST", "/api/procs/sample", &json!({})).await;
    let kind = proc_free(&rig, &["activity", "--kind", "dashboard", "--json"]);
    let json: Value = serde_json::from_str(ok(&kind)).expect("json");
    let rows = json.as_array().expect("activity");
    assert!(!rows.is_empty() && rows.iter().all(|a| a["kind"] == "dashboard"));
    let three = proc_free(&rig, &["activity", "--limit", "3"]);
    assert!(
        ok(&three).lines().count() <= 3 && !three.out.is_empty(),
        "{}",
        three.out
    );
    refused(
        &proc_free(&rig, &["activity", "--limit", "lots"]),
        1,
        "--limit takes a number from 1 to 1000.",
    );
    refused(
        &proc_free(&rig, &["activity", "--limit", "0"]),
        1,
        "--limit takes a number from 1 to 1000.",
    );
    rig.stop().await;
}

fn proc_free(rig: &Rig, argv: &[&str]) -> common::Ran {
    rig.run(argv)
}
