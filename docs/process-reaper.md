<!-- @layer docs @kind doc -->
# Process reaper and a thread's own kill

Two pieces of the process monitor keep the machine free of runs nobody waits on. A thread can
end a process of its own project with `coord proc kill`. The reaper ends, by itself, the
searches and the test or build runs that hung. Both share one scope rule, so neither can touch a
Claude session, the dashboard, another project or the system. This is an Argus feature: the Node
dashboard only lets the Overlord kill.

The evidence is the machine audit of 2026-11-01: two orphaned `find /` held about four million
kernel handles, an orphaned `grep -r /x` walked a whole drive, and a sub-agent's vitest run took
1.7 GB in 8 processes.

The code is `crates/argus-procs/src/scope.rs`, `self_kill.rs` and `reaper/`.

## The scope rule

A project's folders are its root, the folders of its worktree rules, the worktrees the scan
found, the `git worktree list` of its root (read again every 5 minutes) and its ci root
(`coord remote ci-root`). A drive or the file system root never counts as a folder.

A process belongs to every project whose folder holds its working folder, its executable or a
folder its command line names. Where two folders nest, the deeper one wins. A thread of project
P may end a process only when P is the one project it belongs to.

The rule refuses, with a plain sentence that names the reason:

- a Claude session process (`claude.exe`, a node running Claude Code);
- a process of the dashboard itself: the server and its parents, the tray, `argus serve`,
  `argus tray`, an `argus mcp` bridge, and the Node dashboard's own helpers;
- a system process: the protected names (System, csrss, services, svchost, explorer and the
  rest) and anything under the Windows folder;
- a process of another project;
- a process outside every folder of the project;
- a process whose details cannot be read (no folder, no executable, no command line), and a pid
  that is not running.

With `--tree` every process of the tree must pass. One refused process refuses the whole tree.

## A thread's own kill

```text
coord proc kill <pid> [--tree] [--reason "..."]
```

The Overlord keeps its own kill of a listed process (`--reason` required). Any other thread goes
through `POST /api/p/<key>/procs/<pid>/kill {by, tree, reason}`, where `by` must be a live
thread of the board `key`. The monitor takes a fresh sample, reads the working folder of the
process and its tree, checks the scope rule, and ends the process by pid (`taskkill /PID /T /F`
on Windows, never by name). A shadow server refuses.

Each kill or refusal is one activity line of kind `proc-kill`:

```text
thread lead of argus killed C:\node\node.exe 4242 and its tree (3 more): its run hung
refused thread lead of argus the kill of 512: svchost 512 is a system process: ...
```

The CLI prints `killed node.exe 4242 and its tree (3 more).`, or the refusal with exit code 1.

## The reaper

After each sample (every 30 seconds) the reaper looks for two kinds of runs.

1. A search: `find`, `grep`, `rg` or `du` whose parent is gone, or that has run over 10 minutes.
2. A test or build run: `vitest`, `cargo` (with `rustc` under it), `tsc`, `eslint` or `esbuild`,
   as an executable or as the script a node runs. The run is the first such process of its
   chain; the processes under it are its tree. It is ended when the tree gained no CPU time
   (less than 2 seconds) for 5 minutes, or when it is older than the cap (30 minutes).

It never takes `cargo run`, `cargo watch` or `cargo install` (a program the owner may be using),
an esbuild service under a live dev server, a run the dashboard itself started (the runs service
owns those), or a tree the scope rule refuses for the project that owns the run. The owner is the
project of the run's working folder, else of its executable, else of its command line; a run in
no known project folder is left alone.

Each reaped tree is ended by pid with its tree and writes one activity line of kind `proc-reap`:

```text
reaped C:\Git\usr\bin\find.exe 29600: an orphaned search (...) (project tessera)
```

The project's lead gets one `process-reaped` wake per sample that reaped something, and no
second wake within 10 minutes: what it may not hear yet waits for the next sample past that
window. A kill that fails writes one error line and wakes nobody.

## Settings

`config.json` `procs.reaper`:

| Key | Default | Meaning |
| --- | --- | --- |
| `enabled` | `true` | `false` turns the reaper off |
| `shadow` | `false` | write `would reap ...` once per process and kill nothing |
| `capMs` | 1800000 | a test or build run older than this is ended |
| `idleMs` | 300000 | a run whose tree gained no CPU time this long is ended |
| `searchMs` | 600000 | a search running longer than this is ended |
| `wakeDebounceMs` | 600000 | the least time between two wakes to one lead |

A server in shadow mode is always in reaper shadow mode.

## Tests

`crates/argus-procs/tests/monitor/scope.rs`, `self_kill.rs` and `reaper.rs` run the monitor over a
fake process list, a fake killer and the sample's own time as the clock.
`crates/argus-server/tests/procs.rs` covers the route, and `crates/argus-coord/tests/procs_cli.rs`
the verb.
