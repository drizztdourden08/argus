//! The release records of the demo project, in the layout of the release store:
//! `<home>/releases/<project>/<channel>/<version>/release.json` with its notes, a package and an
//! installer. The store reads them as files, so these are plain files.

use std::fs;
use std::io;
use std::path::Path;

use serde_json::json;

/// A release: channel, version, the state of its replay, and its title.
const RELEASES: [(&str, &str, &str, &str); 3] = [
    ("latest", "0.8.0", "done", "Lobby list and join"),
    ("latest", "0.9.0", "pending", "Seats and the host check"),
    ("beta", "0.10.0", "pending", "Chat in the lobby"),
];

fn record(project: &str, (_, version, replay, title): (&str, &str, &str, &str)) -> String {
    let doc = json!({
        "kind": "release", "project": project, "version": version, "tag": format!("v{version}"),
        "commit": "5e3c1a9", "title": format!("{project} {version}: {title}"), "notes": "notes.md",
        "packages": [{ "name": format!("@demo/{project}"), "version": version, "tarball": "package.tgz", "sha512": "demo", "size": 3 }],
        "assets": [{ "name": "Demo Setup.exe", "path": "Demo-Setup.exe", "sha256": "demo", "size": 5, "kind": "installer" }],
        "publishedAt": "2026-11-03T10:00:00Z", "mode": "local",
        "replay": { "state": replay, "at": null, "url": null },
    });
    format!("{doc}\n")
}

fn put(folder: &Path, name: &str, text: &str) -> io::Result<()> {
    fs::write(folder.join(name), text)
}

fn write_one(home: &Path, project: &str, release: (&str, &str, &str, &str)) -> io::Result<()> {
    let (channel, version, _, title) = release;
    let folder = home
        .join("releases")
        .join(project)
        .join(channel)
        .join(version);
    fs::create_dir_all(&folder)?;
    put(&folder, "release.json", &record(project, release))?;
    put(&folder, "notes.md", &format!("# {version}\n\n{title}.\n"))?;
    put(&folder, "package.tgz", "tgz")?;
    put(&folder, "Demo-Setup.exe", "12345")
}

/// Writes the demo releases of `project` under `home`; gives how many.
///
/// # Errors
/// A folder or file cannot be written.
pub fn write(home: &Path, project: &str) -> io::Result<usize> {
    for release in RELEASES {
        write_one(home, project, release)?;
    }
    Ok(RELEASES.len())
}
