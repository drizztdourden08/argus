//! `install-mcp` and `autostart`, ported from `mcp.install.test.mjs`. Everything runs against
//! temp folders (an `APPDATA` of its own, explicit `--config` files); the real Claude Desktop
//! config and Startup folder are never touched. The `coord dashboard` verb and the `dashboard`
//! shims of that file belong to `ai-config`.

use std::fs;
use std::path::{Path, PathBuf};

use argus::install::autostart::{self, VBS_NAME, startup_dir};
use argus::install::code::code_entry;
use argus::install::desktop::{desktop_config_path, desktop_entry};
use argus::install::mcp_config;
use argus::install::self_install::{self, config_line};
use argus::install::vbs::{content, decode, encode};
use argus::install::{Env, Io, Platform};
use serde_json::{Value, json};
use tempfile::TempDir;

const EXE: &str = "X:\\argus\\target\\release\\argus.exe";

fn env_in(tmp: &Path, platform: Platform) -> Env {
    Env {
        platform,
        appdata: Some(tmp.join("appdata")),
        local_appdata: Some(tmp.join("local")),
        xdg_config_home: None,
        home: tmp.join("home"),
        cwd: tmp.to_path_buf(),
        exe: PathBuf::from(EXE),
    }
}

struct Run {
    code: u8,
    out: Vec<String>,
    err: Vec<String>,
}

type Verb = fn(&[String], &mut Io<'_>) -> u8;

fn run(verb: Verb, args: &[&str], env: Env) -> Run {
    let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = {
        let mut say = |l: &str| out.push(l.to_owned());
        let mut warn = |l: &str| err.push(l.to_owned());
        let mut io = Io {
            out: &mut say,
            err: &mut warn,
            env,
        };
        verb(&args, &mut io)
    };
    Run { code, out, err }
}

fn install_mcp(args: &[&str], env: Env) -> Run {
    run(mcp_config::run, args, env)
}

fn tmp() -> TempDir {
    tempfile::tempdir().expect("temp dir")
}

fn json_of(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("file")).expect("json")
}

fn names(dir: &Path) -> Vec<String> {
    let mut all: Vec<String> = fs::read_dir(dir)
        .expect("dir")
        .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
        .collect();
    all.sort();
    all
}

#[test]
fn desktop_prints_the_entry_and_writes_nothing() {
    let dir = tmp();
    let env = env_in(dir.path(), Platform::Windows);
    let config = desktop_config_path(&env);
    let r = install_mcp(&["--desktop"], env);
    assert_eq!(r.code, 0);
    let printed: Value = serde_json::from_str(&r.out.join("\n")).expect("json");
    assert_eq!(printed, desktop_entry(Path::new(EXE)));
    assert_eq!(
        printed["mcpServers"]["coord-dashboard"]["args"],
        json!(["mcp"])
    );
    assert!(!config.exists());
    assert!(
        !dir.path().join("appdata").exists(),
        "not even the folder is created"
    );
    assert!(r.err.join("\n").contains("Not written"));
}

#[test]
fn write_merges_only_our_key_keeps_a_backup_and_is_repeatable() {
    let dir = tmp();
    let folder = dir.path().join("desktop");
    fs::create_dir(&folder).expect("folder");
    let file = folder.join("claude_desktop_config.json");
    let before = json!({ "theme": "dark", "mcpServers": {
        "other": { "command": "x", "args": ["1"] }, "coord-dashboard": { "command": "old" } } });
    fs::write(&file, serde_json::to_string_pretty(&before).expect("json")).expect("seed");
    let args = [
        "--desktop",
        "--write",
        "--config",
        file.to_str().expect("path"),
    ];
    let env = || env_in(dir.path(), Platform::Windows);
    assert_eq!(install_mcp(&args, env()).code, 0);
    let after = json_of(&file);
    assert_eq!(after["theme"], "dark");
    assert_eq!(after["mcpServers"]["other"], before["mcpServers"]["other"]);
    assert_eq!(after["mcpServers"]["coord-dashboard"]["command"], EXE);
    let backups: Vec<String> = names(&folder)
        .into_iter()
        .filter(|f| f.starts_with("claude_desktop_config.json.bak."))
        .collect();
    assert_eq!(backups.len(), 1);
    assert_eq!(
        json_of(&folder.join(&backups[0])),
        before,
        "the backup is the file as it was"
    );
    assert_eq!(install_mcp(&args, env()).code, 0);
    assert_eq!(json_of(&file), after);
}

#[test]
fn write_creates_a_missing_file_without_a_backup_and_refuses_unreadable_json() {
    let dir = tmp();
    let env = || env_in(dir.path(), Platform::Windows);
    let fresh = dir
        .path()
        .join("fresh")
        .join("sub")
        .join("claude_desktop_config.json");
    let args = [
        "--desktop",
        "--write",
        "--config",
        fresh.to_str().expect("path"),
    ];
    assert_eq!(install_mcp(&args, env()).code, 0);
    assert_eq!(json_of(&fresh), desktop_entry(Path::new(EXE)));
    assert_eq!(
        names(fresh.parent().expect("parent")),
        ["claude_desktop_config.json"]
    );

    let broken = dir.path().join("broken.json");
    fs::write(&broken, "{ nope").expect("seed");
    let args = [
        "--desktop",
        "--write",
        "--config",
        broken.to_str().expect("path"),
    ];
    let r = install_mcp(&args, env());
    assert_eq!(r.code, 1);
    assert_eq!(fs::read_to_string(&broken).expect("file"), "{ nope");
    assert!(r.err.join("\n").contains("not valid JSON"));
    assert!(
        names(dir.path())
            .iter()
            .all(|f| !f.starts_with("broken.json.bak"))
    );
}

#[test]
fn the_desktop_config_path_per_platform() {
    let dir = tmp();
    let mut env = env_in(dir.path(), Platform::Windows);
    env.appdata = Some(PathBuf::from("C:\\Users\\me\\AppData\\Roaming"));
    let expected = PathBuf::from("C:\\Users\\me\\AppData\\Roaming")
        .join("Claude")
        .join("claude_desktop_config.json");
    assert_eq!(desktop_config_path(&env), expected);
    env.platform = Platform::Mac;
    assert!(
        desktop_config_path(&env)
            .to_string_lossy()
            .contains("Application Support")
    );
    env.platform = Platform::Other;
    env.xdg_config_home = Some(PathBuf::from("/x"));
    assert_eq!(
        desktop_config_path(&env),
        PathBuf::from("/x")
            .join("Claude")
            .join("claude_desktop_config.json")
    );
}

#[test]
fn usage_errors_exit_2_and_help_exits_0() {
    let dir = tmp();
    for args in [
        vec![],
        vec!["--write"],
        vec!["--bogus"],
        vec!["--desktop", "--nope"],
    ] {
        let r = install_mcp(&args, env_in(dir.path(), Platform::Windows));
        assert_eq!(r.code, 2, "{args:?}");
        assert!(!r.err.is_empty() || !r.out.is_empty());
    }
    assert_eq!(
        install_mcp(&["--help"], env_in(dir.path(), Platform::Windows)).code,
        0
    );
}

#[test]
fn code_takes_the_key_from_the_folder_manifest_or_project_and_matches_the_render() {
    let dir = tmp();
    let project = dir.path().join("proj");
    fs::create_dir(&project).expect("folder");
    fs::write(
        project.join(".ai-config.json"),
        json!({ "projectKey": "rotp" }).to_string(),
    )
    .expect("manifest");
    let env = || env_in(dir.path(), Platform::Windows);
    let r = install_mcp(&["--code", "--dir", project.to_str().expect("path")], env());
    assert_eq!(r.code, 0);
    let printed: Value = serde_json::from_str(&r.out.join("\n")).expect("json");
    assert_eq!(printed, code_entry("rotp", Platform::Windows));
    assert_eq!(
        code_entry("rotp", Platform::Windows)["mcpServers"]["coord-dashboard"],
        json!({ "command": "cmd", "args": ["/c", "dashboard", "mcp", "--project", "rotp"] })
    );
    assert_eq!(
        code_entry("rotp", Platform::Other)["mcpServers"]["coord-dashboard"],
        json!({ "command": "dashboard", "args": ["mcp", "--project", "rotp"] })
    );
    let empty = dir.path().join("empty");
    let given = install_mcp(
        &[
            "--code",
            "--dir",
            empty.to_str().expect("path"),
            "--project",
            "alpha",
        ],
        env(),
    );
    assert_eq!(given.code, 0);
    let none = install_mcp(&["--code", "--dir", empty.to_str().expect("path")], env());
    assert_eq!(none.code, 1);
    assert!(none.err.join("\n").contains("--project"));
    assert_eq!(
        install_mcp(&["--code", "--project", "Bad Key"], env()).code,
        1
    );
}

fn autostart_run(args: &[&str], env: Env) -> Run {
    run(autostart::run, args, env)
}

#[test]
fn the_launcher_text_is_one_hidden_line_with_doubled_quotes() {
    let text = content("C:\\Program Files\\argus\\argus.exe");
    assert_eq!(
        text,
        "CreateObject(\"WScript.Shell\").Run \"\"\"C:\\Program Files\\argus\\argus.exe\"\" tray start\", 0, False\r\n"
    );
}

#[test]
fn a_non_ascii_path_is_written_as_utf16_with_a_mark_and_ascii_stays_plain() {
    let text = "Run \"C:\\Users\\José\\argus.exe\"\r\n";
    let bytes = encode(text);
    assert_eq!(bytes[..2], [0xff, 0xfe]);
    assert_eq!(decode(&bytes), text);
    assert_eq!(encode("plain"), b"plain");
}

#[cfg(windows)]
#[test]
fn install_status_refresh_and_remove_inside_a_temp_startup_folder() {
    let dir = tmp();
    let env = || env_in(dir.path(), Platform::Windows);
    let folder = startup_dir(&env());
    let file = folder.join(VBS_NAME);
    assert!(
        file.to_string_lossy()
            .replace('\\', "/")
            .ends_with("Start Menu/Programs/Startup/coord-dashboard.vbs")
    );

    let r = autostart_run(&["status"], env());
    assert_eq!(r.code, 1);
    assert!(r.out.join("").contains("not installed"));
    assert!(!file.exists());

    assert_eq!(autostart_run(&["install"], env()).code, 0);
    assert_eq!(decode(&fs::read(&file).expect("launcher")), content(EXE));
    assert_eq!(names(&folder), [VBS_NAME], "one file and nothing else");

    let r = autostart_run(&["status"], env());
    assert_eq!(r.code, 0);
    assert!(!r.out.join("").contains("somewhere else"));

    fs::write(&file, "something else").expect("overwrite");
    let r = autostart_run(&["status"], env());
    assert_eq!(r.code, 0);
    assert!(r.out.join("").contains("somewhere else"));

    assert_eq!(autostart_run(&["remove"], env()).code, 0);
    assert!(!file.exists());
    assert_eq!(
        autostart_run(&["remove"], env()).code,
        0,
        "removing twice is fine"
    );
}

#[test]
fn autostart_usage_and_the_platform_guard() {
    let dir = tmp();
    assert_eq!(
        autostart_run(&["bogus"], env_in(dir.path(), Platform::Windows)).code,
        2
    );
    assert_eq!(
        autostart_run(&[], env_in(dir.path(), Platform::Windows)).code,
        2
    );
    let r = autostart_run(&["install"], env_in(dir.path(), Platform::Other));
    assert_eq!(r.code, 1);
    assert!(!dir.path().join("appdata").exists());
}

fn self_install(args: &[&str], env: Env) -> Run {
    run(self_install::run, args, env)
}

/// An env whose running binary is a small file in the temp folder.
fn env_with_exe(tmp: &Path, bytes: &[u8]) -> Env {
    let mut env = env_in(tmp, Platform::Windows);
    env.exe = tmp.join("build").join("argus.exe");
    fs::create_dir_all(env.exe.parent().expect("parent")).expect("build dir");
    fs::write(&env.exe, bytes).expect("exe");
    env
}

fn stable_dir(tmp: &Path) -> PathBuf {
    let dir = tmp.join("stable");
    fs::create_dir_all(&dir).expect("stable dir");
    dir
}

#[test]
fn copies_to_the_dir_prints_the_config_line_and_edits_nothing() {
    let dir = tmp();
    let r = self_install(&["--dir", "stable"], env_with_exe(dir.path(), b"v1"));
    assert_eq!(r.code, 0, "{:?}", r.err);
    let target = dir.path().join("stable").join("argus.exe");
    assert_eq!(fs::read(&target).expect("copy"), b"v1");
    assert_eq!(names(&dir.path().join("stable")), ["argus.exe"]);
    assert!(r.out[0].starts_with("installed "), "{:?}", r.out);
    let line = config_line(&target);
    assert!(r.out[1].ends_with(&line), "{:?}", r.out);
    assert!(line.starts_with("\"exe\": \""));
    assert!(!dir.path().join("appdata").exists());
    assert!(!dir.path().join("local").exists());
}

#[test]
fn the_default_folder_is_programs_argus_under_local_appdata_and_print_copies_nothing() {
    let dir = tmp();
    let r = self_install(&["--print"], env_with_exe(dir.path(), b"v1"));
    assert_eq!(r.code, 0);
    let want = dir.path().join("local").join("Programs").join("argus");
    assert!(r.out[0].contains(&want.join("argus.exe").display().to_string()));
    assert!(!dir.path().join("local").exists(), "--print writes nothing");
}

#[test]
fn a_stale_copy_is_updated_an_equal_one_is_skipped_and_no_temp_file_stays() {
    let dir = tmp();
    let stable = stable_dir(dir.path());
    fs::write(stable.join("argus.exe"), b"old").expect("old");
    let env = env_with_exe(dir.path(), b"new build");
    assert_eq!(self_install(&["--dir", "stable"], env.clone()).code, 0);
    assert_eq!(
        fs::read(stable.join("argus.exe")).expect("file"),
        b"new build"
    );
    let again = self_install(&["--dir", "stable"], env);
    assert_eq!(again.code, 0);
    assert!(
        again.out[0].starts_with("already up to date"),
        "{:?}",
        again.out
    );
    assert_eq!(names(&stable), ["argus.exe"]);
}

#[test]
fn running_from_the_installed_copy_says_so() {
    let dir = tmp();
    let mut env = env_with_exe(dir.path(), b"v1");
    let installed = stable_dir(dir.path()).join("argus.exe");
    fs::copy(&env.exe, &installed).expect("copy");
    env.exe = installed;
    let r = self_install(&["--dir", "stable"], env);
    assert_eq!(r.code, 0);
    assert!(
        r.out[0].starts_with("this is the installed copy"),
        "{:?}",
        r.out
    );
}

#[cfg(windows)]
#[test]
fn a_target_in_use_is_refused_and_left_with_its_folder_as_it_was() {
    use std::os::windows::fs::OpenOptionsExt;
    let dir = tmp();
    let stable = stable_dir(dir.path());
    let target = stable.join("argus.exe");
    fs::write(&target, b"running").expect("running");
    // Share mode 1 shares reads only: no rename or delete, as for a running executable.
    let _held = fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(&target)
        .expect("held");
    let r = self_install(&["--dir", "stable"], env_with_exe(dir.path(), b"new build"));
    assert_eq!(r.code, 1);
    assert!(r.err[0].contains("argus stop"), "{:?}", r.err);
    assert_eq!(fs::read(&target).expect("kept"), b"running");
    assert_eq!(names(&stable), ["argus.exe"]);
}

#[test]
fn self_install_usage_errors_exit_2_and_help_exits_0() {
    let dir = tmp();
    let env = env_with_exe(dir.path(), b"v1");
    assert_eq!(self_install(&["--bogus"], env.clone()).code, 2);
    assert_eq!(self_install(&["--dir"], env.clone()).code, 2);
    assert_eq!(self_install(&["--help"], env).code, 0);
}
