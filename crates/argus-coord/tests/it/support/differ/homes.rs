//! The twin homes: two temp folders, each with a `COORD_HOME`, a `DASHBOARD_HOME` and a work
//! folder (the cwd of every call), seeded with the same boards. The Node call runs on one, the
//! Argus call on the other. A copy of a real board tree is read from its folder and written into
//! a temp home only; the real home itself is never a home of the harness.

use std::fs;
use std::io;
use std::net::TcpListener;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use super::fixture::FixtureBoard;

/// What a home starts with.
#[derive(Clone, Debug)]
pub enum Seed {
    /// Boards built by the fixture builder.
    Fixture(Vec<FixtureBoard>),
    /// A copy of a board tree (`<key>/board.json` and the files beside each board). The
    /// source folder is only read; its `dashboard` folder (database, logs, pid) is not copied.
    CopyOf(PathBuf),
}

/// One temp home.
#[derive(Debug)]
pub struct Home {
    dir: TempDir,
}

impl Home {
    fn create() -> io::Result<Home> {
        let dir = TempDir::new()?;
        let home = Home { dir };
        fs::create_dir_all(home.dashboard())?;
        fs::create_dir_all(home.work())?;
        Ok(home)
    }

    /// The temp folder that holds the whole home.
    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    /// `COORD_HOME`: one folder per project, each with its `board.json`.
    pub fn coord(&self) -> PathBuf {
        self.root().join("coord")
    }

    /// `DASHBOARD_HOME`, inside `COORD_HOME` as in the real layout.
    pub fn dashboard(&self) -> PathBuf {
        self.coord().join("dashboard")
    }

    /// The cwd of every call, and the root of seeded threads.
    pub fn work(&self) -> PathBuf {
        self.root().join("work")
    }

    /// The path of a project's `board.json`.
    pub fn board(&self, project: &str) -> PathBuf {
        self.coord().join(project).join("board.json")
    }

    fn seed(&self, seed: &Seed, port: u16) -> io::Result<()> {
        match seed {
            Seed::Fixture(boards) => self.write_fixture(boards)?,
            Seed::CopyOf(source) => copy_tree(source, &self.coord(), true)?,
        }
        let config = format!("{{\n  \"port\": {port}\n}}\n");
        fs::write(self.dashboard().join("config.json"), config)
    }

    fn write_fixture(&self, boards: &[FixtureBoard]) -> io::Result<()> {
        let work = self.work().to_string_lossy().into_owned();
        for board in boards {
            let file = self.board(board.project());
            fs::create_dir_all(file.parent().expect("board folder"))?;
            fs::write(file, board.to_json(&work))?;
        }
        Ok(())
    }
}

/// Two homes seeded alike, and the closed port both point their dashboard config at.
#[derive(Debug)]
pub struct Twin {
    /// The home the Node engine runs on.
    pub node: Home,
    /// The home the Argus engine runs on.
    pub rust: Home,
    /// A loopback port nothing listens on: no live dashboard is ever reached.
    pub closed_port: u16,
}

impl Twin {
    /// Creates and seeds both homes.
    pub fn seeded(seed: &Seed) -> io::Result<Twin> {
        let closed_port = closed_port()?;
        let (node, rust) = (Home::create()?, Home::create()?);
        node.seed(seed, closed_port)?;
        rust.seed(seed, closed_port)?;
        Ok(Twin {
            node,
            rust,
            closed_port,
        })
    }
}

/// A port that was free a moment ago: bound, read, and released.
fn closed_port() -> io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

/// Copies the files under `from` into `to`. At the top level the `dashboard` folder is
/// skipped; a board lock left behind is never copied.
fn copy_tree(from: &Path, to: &Path, top: bool) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let name = entry.file_name();
        let skip = (top && name == "dashboard") || name == "board.lock";
        if skip {
            continue;
        }
        let target = to.join(&name);
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target, false)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
