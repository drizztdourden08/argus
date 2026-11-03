//! The release store and its feed on a temp folder: the index, half written entries, the made
//! `latest.yml`, a channel level file that wins, and what the feed refuses.
//! Keep rule: K2, K4.
// test-tag: standard

use std::fs;
use std::path::Path;

use argus_core::fixture::put_file as put;
use argus_records::releases::{ReleaseStore, Served};
use serde_json::{Value, json};

const LATEST: &str = "version: VER\nfiles:\n  - url: App-Setup-VER.exe\n    sha512: abc==\n    size: 5\npath: App-Setup-VER.exe\nsha512: abc==\n";

fn record(version: &str, replay: &str) -> String {
    json!({
        "kind": "release", "project": "brock", "version": version, "tag": format!("v{version}"),
        "commit": "abc1234", "title": format!("Brock {version}"), "notes": "notes.md",
        "packages": [{ "name": "@d/core", "version": version, "tarball": "core.tgz", "sha512": "x", "size": 3 }],
        "assets": [{ "name": "App Setup.exe", "path": "App-Setup.exe", "sha256": "y", "size": 5, "kind": "installer" }],
        "publishedAt": "2026-10-29T10:00:00Z", "mode": "local", "replay": { "state": replay, "at": null, "url": null },
    })
    .to_string()
}

fn release(root: &Path, channel: &str, version: &str, replay: &str, with_yml: bool) {
    let base = format!("brock/{channel}/{version}");
    put(
        root,
        &format!("{base}/release.json"),
        record(version, replay),
    );
    put(root, &format!("{base}/notes.md"), "# notes");
    put(root, &format!("{base}/core.tgz"), "tgz");
    put(root, &format!("{base}/App-Setup.exe"), "12345");
    if with_yml {
        put(
            root,
            &format!("{base}/latest.yml"),
            LATEST.replace("VER", version),
        );
    }
}

fn store(dir: &tempfile::TempDir) -> ReleaseStore {
    let s = ReleaseStore::new(&dir.path().join("releases"));
    s.rescan();
    s
}

fn text(served: Option<Served>) -> String {
    match served {
        Some(Served::Text(t)) => t,
        other => panic!("not a made manifest: {other:?}"),
    }
}

#[test]
fn the_index_lists_versions_newest_first_with_files_and_replay() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().join("releases");
    release(&root, "latest", "1.9.0", "done", true);
    release(&root, "latest", "1.10.0", "pending", true);
    let s = store(&dir);
    let channels = s.channels_json("brock");
    assert_eq!(channels[0]["channel"], "latest");
    assert_eq!(channels[0]["latest"], "1.10.0");
    let versions: Vec<&Value> = channels[0]["versions"]
        .as_array()
        .expect("versions")
        .iter()
        .collect();
    assert_eq!(versions[0]["version"], "1.10.0");
    assert_eq!(versions[1]["version"], "1.9.0");
    assert_eq!(
        versions[0]["notesUrl"],
        "/releases/brock/latest/1.10.0/notes.md"
    );
    assert_eq!(
        versions[0]["assets"][0]["url"],
        "/releases/brock/latest/1.10.0/App-Setup.exe"
    );
    assert_eq!(
        versions[0]["packages"][0]["url"],
        "/releases/brock/latest/1.10.0/core.tgz"
    );
    assert_eq!(versions[0]["replay"]["state"], "pending");
    assert_eq!(s.pending(), 1);
    assert_eq!(s.projects_json()[0]["project"], "brock");
}

#[test]
fn a_release_appears_when_its_record_is_written_and_half_written_entries_do_not() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().join("releases");
    let s = store(&dir);
    assert!(s.releases().is_empty());
    put(&root, "brock/latest/1.0.0/notes.md", "no record yet");
    put(
        &root,
        "brock/latest/1.1.0.tmp/release.json",
        record("1.1.0", "pending"),
    );
    put(&root, "brock/latest/1.2.0/release.json", "{ half");
    s.rescan();
    assert!(
        s.releases().is_empty(),
        "no record, a temp folder, a torn record"
    );
    put(
        &root,
        "brock/latest/1.2.0/release.json",
        record("1.2.0", "pending"),
    );
    s.rescan();
    assert_eq!(s.releases().len(), 1);
    assert_eq!(s.releases()[0].version, "1.2.0");
}

#[test]
fn the_channel_manifest_is_made_from_the_newest_release_with_urls_into_its_folder() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().join("releases");
    release(&root, "latest", "1.9.0", "done", true);
    release(&root, "latest", "1.10.0", "done", true);
    release(&root, "latest", "1.11.0-beta.1", "done", true);
    release(&root, "latest", "1.12.0", "done", false);
    let s = store(&dir);
    let made = text(s.serve("brock/latest/latest.yml"));
    assert!(made.starts_with("version: 1.10.0\n"), "{made}");
    assert!(made.contains("  - url: 1.10.0/App-Setup-1.10.0.exe\n"));
    assert!(made.contains("\npath: 1.10.0/App-Setup-1.10.0.exe\n"));
    assert_eq!(s.latest("brock", "latest").as_deref(), Some("1.10.0"));
    assert_eq!(s.serve("brock/latest/latest-mac.yml"), None);
}

#[test]
fn a_channel_of_prereleases_falls_back_to_the_newest_of_them() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().join("releases");
    release(&root, "beta", "2.0.0-beta.1", "done", true);
    release(&root, "beta", "2.0.0-beta.2", "done", true);
    let s = store(&dir);
    assert_eq!(s.latest("brock", "beta").as_deref(), Some("2.0.0-beta.2"));
}

#[test]
fn a_manifest_on_disk_at_the_channel_level_wins() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().join("releases");
    release(&root, "latest", "1.10.0", "done", true);
    put(
        &root,
        "brock/latest/latest.yml",
        "version: 1.9.0\npath: 1.9.0/x.exe\n",
    );
    let s = store(&dir);
    let served = s.serve("brock/latest/latest.yml");
    assert!(matches!(served, Some(Served::File(_))));
    assert_eq!(s.latest("brock", "latest").as_deref(), Some("1.9.0"));
}

#[test]
fn files_are_served_and_everything_that_leaves_the_store_or_lists_it_is_refused() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().join("releases");
    release(&root, "latest", "1.0.0", "done", true);
    fs::write(dir.path().join("secret.txt"), "no").expect("outside");
    let s = store(&dir);
    assert!(matches!(
        s.serve("brock/latest/1.0.0/App-Setup.exe"),
        Some(Served::File(_))
    ));
    for bad in [
        "brock/latest/1.0.0",
        "brock/latest",
        "brock",
        "",
        "../secret.txt",
        "brock/../../secret.txt",
        "brock/latest/1.0.0/../../../../secret.txt",
        "/brock/latest/1.0.0/App-Setup.exe",
        "brock\\latest\\1.0.0\\App-Setup.exe",
        "brock/latest/1.0.0/missing.exe",
    ] {
        assert_eq!(s.serve(bad), None, "{bad}");
    }
}

fn git(repo: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .output()
        .expect("git runs");
    assert!(out.status.success(), "git {args:?}: {out:?}");
    String::from_utf8(out.stdout)
        .expect("utf8")
        .trim()
        .to_owned()
}

fn commit(repo: &Path, subject: &str) -> String {
    git(repo, &["commit", "--allow-empty", "-m", subject]);
    git(repo, &["rev-parse", "HEAD"])
}

fn release_at(root: &Path, version: &str, sha: &str) {
    let base = format!("brock/latest/{version}");
    put(
        root,
        &format!("{base}/release.json"),
        record(version, "done").replace("abc1234", sha),
    );
}

#[test]
fn a_version_lists_the_items_and_merged_prs_of_the_commits_since_the_previous_one() {
    let dir = tempfile::tempdir().expect("temp dir");
    let repo = dir.path().join("repo");
    fs::create_dir_all(&repo).expect("repo folder");
    git(&repo, &["init", "-q"]);
    let first = commit(&repo, "Start the pad (I-1)");
    commit(&repo, "Fix I-2 and I-3 together");
    let second = commit(&repo, "Pad overlay for I-7 (#5)");
    put(
        dir.path(),
        "remote.json",
        json!({ "mode": "local", "ciRoots": { "brock": repo.to_string_lossy() } }).to_string(),
    );
    let root = dir.path().join("releases");
    release_at(&root, "1.0.0", &first);
    release_at(&root, "1.1.0", &second);
    let s = store(&dir);
    let versions = s.channels_json("brock")[0]["versions"].clone();
    assert_eq!(versions[0]["version"], "1.1.0");
    assert_eq!(versions[0]["items"], json!(["I-2", "I-3", "I-7"]));
    assert_eq!(
        versions[0]["prs"],
        json!([{ "number": 5, "commit": second }])
    );
    assert_eq!(
        versions[1]["items"],
        json!(["I-1"]),
        "no earlier version: the history up to it"
    );
    assert_eq!(versions[1]["prs"], json!([]));
}

#[test]
fn a_version_has_empty_items_and_prs_when_the_project_has_no_checkout() {
    let dir = tempfile::tempdir().expect("temp dir");
    release(
        &dir.path().join("releases"),
        "latest",
        "1.0.0",
        "done",
        true,
    );
    let versions = store(&dir).channels_json("brock")[0]["versions"].clone();
    assert_eq!(versions[0]["items"], json!([]));
    assert_eq!(versions[0]["prs"], json!([]));
}

#[test]
fn an_old_record_without_a_commit_is_listed_with_empty_items() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().join("releases");
    put(
        &root,
        "brock/latest/0.1.0/release.json",
        json!({ "kind": "release", "project": "brock", "version": "0.1.0" }).to_string(),
    );
    let versions = store(&dir).channels_json("brock")[0]["versions"].clone();
    assert_eq!(versions[0]["items"], json!([]));
}
