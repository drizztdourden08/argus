//! Measuring a leftover folder: read-only and bounded.
//!
//! Breadth first, so the top of the tree is seen first. `.git` is never entered (one below the top is a
//! nested repository). `node_modules` is counted by its top-level entries only. A link is a leaf that is
//! never followed; its own modification time counts. A folder that did not fit the bounds is `partial`.

use crate::fs::{Entry, Fs};
use crate::paths::join_path;

pub const MEASURE_MAX_ENTRIES: u64 = 20_000;
pub const MEASURE_MAX_MS: i64 = 4000;
pub const TOP_ENTRIES: usize = 8;

/// The limits of one measuring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub max_entries: u64,
    pub max_ms: i64,
}

impl Bounds {
    /// The scan's bounds.
    pub const SCAN: Self = Self {
        max_entries: MEASURE_MAX_ENTRIES,
        max_ms: MEASURE_MAX_MS,
    };
    /// The look right before a deletion is wider: it is the last chance to see a nested repository or a young file.
    pub const REMOVE: Self = Self {
        max_entries: 10 * MEASURE_MAX_ENTRIES,
        max_ms: 5 * MEASURE_MAX_MS,
    };
}

/// What a leftover folder holds.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Leftover {
    pub size_mb: Option<f64>,
    pub files: Option<u64>,
    pub newest_ms: Option<i64>,
    pub has_nested_git: bool,
    pub nested_git: Option<String>,
    pub node_modules: bool,
    pub partial: bool,
    pub is_link: bool,
    pub pending: bool,
    pub top_entries: Vec<String>,
    pub entries: u64,
    pub measured_at: Option<i64>,
}

impl Leftover {
    /// What a folder that was not measured yet looks like (the scan's budget ran out).
    pub fn pending() -> Self {
        Self {
            partial: true,
            pending: true,
            ..Self::default()
        }
    }
}

struct Walk<'a> {
    fs: &'a dyn Fs,
    bounds: Bounds,
    now: &'a dyn Fn() -> i64,
    started: i64,
    out: Leftover,
    bytes: u64,
    seen: u64,
    files: u64,
}

struct Pending {
    dir: String,
    rel: String,
    items: Option<Vec<Entry>>,
}

enum Flow {
    Next,
    Stop,
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

impl Walk<'_> {
    fn touch(&mut self, mtime_ms: Option<i64>) {
        if let Some(t) = mtime_ms {
            if self.out.newest_ms.is_none_or(|n| t > n) {
                self.out.newest_ms = Some(t);
            }
        }
    }

    fn over_bounds(&mut self) -> bool {
        self.seen += 1;
        let over =
            self.seen > self.bounds.max_entries || (self.now)() - self.started > self.bounds.max_ms;
        if over {
            self.out.partial = true;
        }
        over
    }

    fn count_node_modules(&mut self, path: &str) {
        self.out.node_modules = true;
        match self.fs.read_dir(path) {
            Ok(names) => {
                let n = u64::try_from(names.len()).unwrap_or(u64::MAX);
                self.files += n;
                self.seen += n;
            }
            Err(_) => self.out.partial = true,
        }
    }

    fn visit(&mut self, dir: &str, rel: &str, entry: &Entry, queue: &mut Vec<Pending>) -> Flow {
        if self.over_bounds() {
            return Flow::Stop;
        }
        if entry.name == ".git" {
            self.out.has_nested_git = true;
            if self.out.nested_git.is_none() {
                self.out.nested_git = Some(if rel.is_empty() {
                    ".git".to_string()
                } else {
                    format!("{rel}/.git")
                });
            }
            return Flow::Next;
        }
        let path = join_path(dir, &entry.name);
        let Ok(stat) = self.fs.lstat(&path) else {
            return Flow::Next;
        };
        self.touch(stat.mtime_ms);
        if entry.is_link || stat.is_link {
            self.files += 1;
        } else if stat.is_dir && entry.name == "node_modules" {
            self.count_node_modules(&path);
        } else if stat.is_dir {
            let rel = if rel.is_empty() {
                entry.name.clone()
            } else {
                format!("{rel}/{}", entry.name)
            };
            queue.push(Pending {
                dir: path,
                rel,
                items: None,
            });
        } else {
            self.files += 1;
            self.bytes += stat.size;
        }
        Flow::Next
    }

    fn items_of(&mut self, pending: &mut Pending) -> Option<Vec<Entry>> {
        if let Some(items) = pending.items.take() {
            return Some(items);
        }
        let read = self.fs.read_dir(&pending.dir);
        if read.is_err() {
            self.out.partial = true;
        }
        read.ok()
    }

    fn walk(&mut self, mut queue: Vec<Pending>) {
        let mut at = 0;
        while at < queue.len() {
            let mut pending = std::mem::replace(
                &mut queue[at],
                Pending {
                    dir: String::new(),
                    rel: String::new(),
                    items: None,
                },
            );
            at += 1;
            let Some(items) = self.items_of(&mut pending) else {
                continue;
            };
            let (dir, rel) = (pending.dir, pending.rel);
            let mut found = Vec::new();
            for entry in &items {
                if matches!(self.visit(&dir, &rel, entry, &mut found), Flow::Stop) {
                    return;
                }
            }
            queue.append(&mut found);
        }
    }

    fn finish(mut self) -> Leftover {
        #[allow(clippy::cast_precision_loss)]
        let mb = self.bytes as f64 / 1_048_576.0;
        self.out.size_mb = Some(round1(mb));
        self.out.files = Some(self.files);
        self.out.entries = self.seen.min(self.bounds.max_entries);
        self.out
    }
}

fn top_names(top: &[Entry]) -> Vec<String> {
    let mut names: Vec<String> = top
        .iter()
        .map(|e| {
            if e.is_dir {
                format!("{}/", e.name)
            } else {
                e.name.clone()
            }
        })
        .collect();
    names.sort();
    names.truncate(TOP_ENTRIES);
    names
}

/// Measure a folder: totals, the newest change, the top entries, nested repositories, `node_modules`, links.
pub fn measure_folder(fs: &dyn Fs, root: &str, bounds: Bounds, now: &dyn Fn() -> i64) -> Leftover {
    let started = now();
    let out = Leftover {
        measured_at: Some(started),
        ..Leftover::default()
    };
    let mut walk = Walk {
        fs,
        bounds,
        now,
        started,
        out,
        bytes: 0,
        seen: 0,
        files: 0,
    };
    let top = match fs.lstat(root) {
        Ok(stat) => {
            walk.touch(stat.mtime_ms);
            if stat.is_link {
                walk.out.is_link = true;
                return walk.finish();
            }
            fs.read_dir(root)
        }
        Err(err) => Err(err),
    };
    let Ok(top) = top else {
        walk.out.partial = true;
        return walk.finish();
    };
    walk.out.top_entries = top_names(&top);
    walk.walk(vec![Pending {
        dir: root.to_string(),
        rel: String::new(),
        items: Some(top),
    }]);
    walk.finish()
}

/// 1536 -> `1.5 GB`, 300 -> `300 MB`, 0.2 -> `under 1 MB`, none -> `unknown size`.
pub fn size_text(mb: Option<f64>) -> String {
    let Some(mb) = mb.filter(|m| m.is_finite()) else {
        return "unknown size".to_string();
    };
    if mb < 1.0 {
        return "under 1 MB".to_string();
    }
    if mb < 1024.0 {
        return format!("{} MB", mb.round());
    }
    let gb = (mb / 1024.0 * 10.0).round() / 10.0;
    if (gb - gb.round()).abs() < f64::EPSILON {
        format!("{} GB", gb.round())
    } else {
        format!("{gb:.1} GB")
    }
}

/// 300 -> `about 300 MB`, 0.2 -> `under 1 MB`, none -> `unknown size`.
pub fn about_text(mb: Option<f64>) -> String {
    match mb {
        Some(m) if m.is_finite() && m >= 1.0 => format!("about {}", size_text(mb)),
        _ => size_text(mb),
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::{RealFs, Stat};
    use crate::testkit::{backdate, put, root_of};
    use std::cell::Cell;
    use std::io;
    use tempfile::tempdir;

    const MB: usize = 1024 * 1024;

    fn put_bytes(root: &str, rel: &str, size: usize) {
        put(root, rel, &"x".repeat(size));
    }

    fn real_now() -> i64 {
        crate::clock::Clock::now_ms(&crate::clock::SystemClock)
    }

    #[test]
    fn files_size_newest_top_entries_node_modules_and_a_nested_repository() {
        let dir = tempdir().unwrap();
        let root = root_of(&dir);
        put(&root, "a.txt", &"a".repeat(100));
        put_bytes(&root, "docs/b.bin", MB);
        put(&root, "docs/deep/c.txt", "c");
        for p in ["p1", "p2", "p3"] {
            put(
                &root,
                &format!("node_modules/{p}/index.js"),
                "module.exports = 1",
            );
        }
        put(&root, "vendor/repo/.git/HEAD", "ref: refs/heads/main\n");
        put(&root, "vendor/repo/src.txt", "code");
        backdate(&root, 5.0);
        let m = measure_folder(&RealFs, &root, Bounds::SCAN, &real_now);
        assert_eq!(
            (
                m.files,
                m.node_modules,
                m.has_nested_git,
                m.partial,
                m.is_link
            ),
            (Some(7), true, true, false, false)
        );
        assert_eq!(m.nested_git.as_deref(), Some("vendor/repo/.git"));
        assert_eq!(
            m.size_mb,
            Some(1.0),
            "about 1 MB, node_modules is not counted"
        );
        let age = real_now() - m.newest_ms.unwrap();
        assert!(
            (age - 5 * 86_400_000).abs() < 120_000,
            "the newest modification time found: {age}"
        );
        assert_eq!(
            m.top_entries,
            ["a.txt", "docs/", "node_modules/", "vendor/"]
        );
    }

    struct Virtual {
        wide: bool,
    }

    impl Fs for Virtual {
        fn exists(&self, _: &str) -> bool {
            true
        }
        fn is_dir(&self, _: &str) -> bool {
            true
        }
        fn dirs(&self, _: &str) -> io::Result<Vec<String>> {
            Ok(Vec::new())
        }
        fn mtime_ms(&self, _: &str) -> Option<i64> {
            Some(5000)
        }
        fn lstat(&self, path: &str) -> io::Result<Stat> {
            Ok(Stat {
                is_link: false,
                is_dir: !self.wide || path == "X:/fake",
                mtime_ms: Some(5000),
                size: 10,
            })
        }
        fn read_dir(&self, _: &str) -> io::Result<Vec<Entry>> {
            if self.wide {
                return Ok((0..50_000)
                    .map(|i| Entry {
                        name: format!("f{i}"),
                        is_dir: false,
                        is_link: false,
                    })
                    .collect());
            }
            Ok(["a", "b", "c"]
                .map(|n| Entry {
                    name: n.to_string(),
                    is_dir: true,
                    is_link: false,
                })
                .to_vec())
        }
        fn rename(&self, _: &str, _: &str) -> io::Result<()> {
            Ok(())
        }
        fn remove_file(&self, _: &str) -> io::Result<()> {
            Ok(())
        }
        fn remove_dir(&self, _: &str) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn the_bounds_stop_a_big_tree_at_the_entry_limit_and_at_the_time_limit() {
        let wide = Virtual { wide: true };
        let a = measure_folder(
            &wide,
            "X:/fake",
            Bounds {
                max_entries: 1000,
                max_ms: 4000,
            },
            &|| 0,
        );
        assert_eq!((a.partial, a.entries, a.files), (true, 1000, Some(1000)));
        let deep = Virtual { wide: false };
        let b = measure_folder(
            &deep,
            "X:/fake",
            Bounds {
                max_entries: 500,
                max_ms: 4000,
            },
            &|| 0,
        );
        assert_eq!((b.partial, b.entries), (true, 500));
        let ticks = Cell::new(0);
        let clock = || {
            ticks.set(ticks.get() + 1);
            ticks.get()
        };
        let c = measure_folder(
            &wide,
            "X:/fake",
            Bounds {
                max_entries: u64::MAX,
                max_ms: 40,
            },
            &clock,
        );
        assert!(c.partial);
        assert!(
            c.files.unwrap() < 60 && c.files.unwrap() > 5,
            "stopped by the clock after {:?} entries",
            c.files
        );
        let roomy = measure_folder(
            &wide,
            "X:/fake",
            Bounds {
                max_entries: 100_000,
                max_ms: 4000,
            },
            &|| 0,
        );
        assert!(!roomy.partial, "inside both bounds");
        assert_eq!(roomy.files, Some(50_000));
    }

    #[test]
    fn size_text_reads_plainly() {
        let got: Vec<String> = [
            Some(0.2),
            Some(1.0),
            Some(300.0),
            Some(1536.0),
            Some(6144.0),
            Some(2048.0),
            None,
        ]
        .into_iter()
        .map(size_text)
        .collect();
        assert_eq!(
            got,
            [
                "under 1 MB",
                "1 MB",
                "300 MB",
                "1.5 GB",
                "6 GB",
                "2 GB",
                "unknown size"
            ]
        );
        assert_eq!(about_text(Some(300.0)), "about 300 MB");
        assert_eq!(about_text(Some(0.2)), "under 1 MB");
    }
}
