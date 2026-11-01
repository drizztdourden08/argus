<!-- @layer docs @kind doc -->
# Argus architecture

Argus is the Rust and React rewrite of the Node coord dashboard in `X:/ai-config/dashboard`.
The Node `DESIGN.md` in that folder is the specification and its `test` folder is the proof.
This page says how the rewrite is built: the crates, the data model, the wire contract that
stays byte-compatible, the services, the web app and the switch-over. `docs/parity.md` says
how each behaviour is proven and in which slice it lands.

Section numbers such as "DESIGN 13.7" point at the Node `DESIGN.md`.

## Ground rules

- One Cargo workspace under `crates/`, one binary `argus`, one web app in `apps/web`.
- The HTTP API, the SSE streams, the MCP tools and the data folder stay compatible with the
  Node server, so the `coord` CLI, the session hooks, `dashboard wake-stream` and the MCP
  clients keep working at switch-over without a change.
- The Node server runs until Argus passes the ported behaviour tests. Nothing the owner uses
  regresses on the day Argus takes port 47615.
- The web app is rendered only with Tessera. A missing piece is a request to the Tessera lead.
- A behaviour that needs a change in `ai-config` (the `coord` CLI, the hooks, the render) is a
  request to the Overlord through `coord lead portfolio`, never a local patch.
- The specification still moves: the Node dashboard gains features until the switch-over. These
  pages are written against `ai-config` commit `4dbe260` (board template 45, Overlord page 17,
  six migrations, 23 MCP tools). Every later Node commit that touches `dashboard/` or the coord
  CLI adds or updates rows in `docs/parity.md` before it is ported.

## Code rules

The Rust lane runs beside the TypeScript gates and Brock enforces both.

| Gate | Command or rule |
|---|---|
| Format | `cargo fmt --check` |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` |
| Tests | `cargo test --workspace` |
| Function size | 60 lines at most (`too-many-lines-threshold = 60` in `clippy.toml`, `clippy::too_many_lines` denied) |
| Complexity | cognitive complexity 10 at most (`cognitive-complexity-threshold = 10`, `clippy::cognitive_complexity` denied) |
| Comments | only `///` and `//!` doc comments; a `// SAFETY:` line above each `unsafe` block |
| Duplication | `jscpd` scans `.rs` files too: two blocks alike become one function |
| Structure | one concern per module, small functions, no dead code |
| Web | `pnpm lint`, `pnpm lint:md`, `pnpm structure`, `pnpm test` |

The Brock cargo lane runs when a root `Cargo.toml` exists. A rule that blocks a sound design goes
to the Brock lead with `coord lead brock "..."`.

Integration tests are one binary per crate: every file is a module of
`crates/<crate>/tests/it/main.rs`, so a crate links its tests once. Run one file with
`cargo test -p <crate> --test it <module>::`. A test that must run in its own process stays a
top-level file in `tests/` (`argus-coord`: `latency_bench`, the ignored bench).

### Rust test tags and the runner

Every Rust file that holds a `#[test]` or `#[tokio::test]` carries exactly one tag line, as in the
owner's test standard (`X:/ai-config/docs/test-standard.md`):

| Tag | Line | Runs |
|---|---|---|
| critical | `// test-tag: critical` | every `pnpm test`, for the changed files only |
| standard | `// test-tag: standard` | `pnpm test:pr`, critical plus standard |
| e2e | `// test-tag: e2e` | `pnpm test:e2e`; each test also has `#[ignore = "e2e"]` |

The line is the first line after the `//!` header block of a test file, or the line above
`#[cfg(test)]` for an inline `mod tests`. A file that mixes kinds is split in two.

Parity tests (the `argus-coord` remote differ and anything that compares Node with Rust,
`argus-parity`) are e2e. They run only before the daily sync, never in a plain `cargo test`.

`scripts/rust-tests.mjs` runs only the tests that cover the changed files. It never runs a whole
crate by default, never the whole workspace, and runs one `cargo test -p <crate> -j 3` at a time.

```text
node scripts/rust-tests.mjs --tag critical|standard|e2e --changed [--base <ref>] [--dry-run]
node scripts/rust-tests.mjs --tag critical|standard|e2e --files <file>... [--dry-run]
node scripts/rust-tests.mjs --check-tags
```

- `--changed` takes the files touched since `git merge-base HEAD main` (or `--base`), plus
  uncommitted and untracked files.
- A changed test file runs its own module. A changed helper file (no tests) runs the tests of its
  binary that use the module. A changed source file runs every tagged test of its crate and of the
  crates that depend on it directly (`--depth <n>` goes further, default 1). A changed crate
  `Cargo.toml` runs the tagged tests of that crate only.
- Module paths follow the `mod` declarations, `#[path]` included: `tests/it/<x>.rs` is `<x>::` of
  `--test it`, `src/specs/<x>.rs` is `specs::<x>::` of `--lib`, an inline `mod tests` in
  `src/<a>/<b>.rs` is `a::b::tests::`, and `tests/<name>.rs` is its own `--test <name>`.
- `--ignored` and `--test-threads=1` are added for e2e. Filters are substring matches, so the runner
  adds `--skip` for a module of another tag that a filter would also match.
- A file with no tag line or two tag lines is refused (exit 1). `--check-tags` lists every Rust file
  with tests and not exactly one tag, and every e2e file with fewer `#[ignore]` than tests; it only
  reads files. It is the Rust half of the lint rule `local/test-has-one-tag`.

## Workspace layout

```text
Cargo.toml                  workspace, shared lints, profile settings
clippy.toml                 thresholds above
crates/
  argus/                    the binary: serve, start, stop, status, open, logs, resync, token,
                            wake-stream, mcp, tray, autostart, install-mcp, later coord
  argus-core/               shared types, config, paths, ids, clock, bus, redaction, logging
  argus-dns/                the tiny DNS responder of the local names, and a client to ask it
  argus-store/              SQLite, migrations, DocStore, kv, activity log
  argus-board/              board.json model: read, normalize, lock, save, ops, projections
  argus-sync/               board watcher, exporter, owner posts, portfolio cards, proofs
  argus-server/             axum HTTP, security checks, API routes, SSE, embedded web assets
  argus-wake/               wake policy, engine, listeners, registry, faults, presence, timers
  argus-procs/              process rules, status, alerts, kills
  argus-worktrees/          worktree rules, read-only git door, scans, removal, leftovers
  argus-cache/              Brock's build cache: read it, who uses each entry, prune (docs/build-cache.md)
  argus-launchers/          run defs, runs, spawn, kill, logs, attachments, launchers
  argus-agents/             sub-agent transcript watcher, reaper, reconciler
  argus-notify/             notification model, store, fan-out
  argus-native/             Windows tray, toasts, native process sampling
  argus-mcp/                MCP tool host, JSON-RPC, stdio bridge
  argus-testkit/            test-only: temp homes, fixture boards, test server, SSE reader
  argus-coord/              slice 8: the coord verbs and hooks in Rust
apps/
  web/                      Vite, React, TypeScript, Tessera
docs/
  architecture.md, parity.md
```

## Crates

The ten crates of the scaffold keep their names. Six crates are added, each for a reason given
in its row.

| Crate | Responsibility | Depends on |
|---|---|---|
| `argus` | Binary entry, `clap` subcommands, boot and shutdown order | server, mcp, native, core |
| `argus-core` (added) | Types every crate shares, so no service crate depends on another for a struct | none |
| `argus-dns` (added) | The responder of the local names (`.argus`): UDP and TCP, one address, no recursion | none |
| `argus-store` | The one SQLite connection, migrations, docs and changes, kv, activity | core |
| `argus-board` (added) | Everything about a board file, shared by the server now and the Rust coord later | core |
| `argus-sync` (added) | Mirrors boards into docs and applies owner posts, the Node `src/sync` | core, store, board |
| `argus-server` | Composes the services, serves HTTP and SSE, embeds the web build | every service crate |
| `argus-wake` | Wakes, listeners, registry, faults, presence, agent and item reminders | core, store, board |
| `argus-procs` | The process monitor, the Overlord's rules, alerts and kills | core, store, wake, native |
| `argus-worktrees` | The Worktrees tab backend: scans, statuses, stale wakes, removal | core, store, board, wake, procs |
| `argus-cache` (added) | The build cache of Brock and Tessera: entries, lockfile users, the prune rule | core, worktrees (its junction-safe file system) |
| `argus-launchers` | Runs and launchers: defs, lifecycle, spawn, logs, stdin, attachments, launcher wakes | core, store, board, wake |
| `argus-agents` (added) | Sub-agent liveness, three services with one data source | core, store, board, wake |
| `argus-notify` | Notifications to the owner: record, limits, automatic pings, toast with tray fallback | core, store, board, native |
| `argus-native` | Windows APIs only: tray, toasts, process sampling, single-instance mutex | core |
| `argus-mcp` | MCP tools and resources, JSON-RPC handling, the stdio bridge | core, store, board, sync, launchers, wake |
| `argus-testkit` (added) | One harness for every integration test, a dev-dependency only | server, store, core |
| `argus-coord` (added, slice 8) | The coord CLI port behind the `coord` shim | core, board, agents, launchers, worktrees, notify (their pure modules); no service crate depends on it |

The dependency graph has no cycle. Service crates talk to each other through the bus in
`argus-core` (events) and through traits that `argus-board` and `argus-wake` define (calls).

```text
argus
  -> argus-server -> argus-mcp -> argus-sync -> argus-board -> argus-core
                  -> argus-wake, argus-procs, argus-worktrees, argus-launchers,
                     argus-agents, argus-notify -> argus-store -> argus-core
  -> argus-native -> argus-core
argus-procs -> argus-native (sampling), argus-notify -> argus-native (toasts)
argus-launchers, argus-procs, argus-worktrees, argus-agents -> argus-wake (submit)
```

### argus-core

Public surface:

```rust
pub type Scope = String;
pub enum Coll { Threads, Items, Tests, Entries, Locks, Actions, Agents, Log, Meta, Runs, Launches,
    Cards, Builds, Launchers, Wakes, Registry, Worktrees, Procs, Activity, Notifications }
pub struct DocChange { pub coll: Coll, pub id: String, pub op: DocOp, pub data: Option<Value> }
pub struct AppliedChange { pub change: DocChange, pub seq: i64 }
pub struct PostResult { pub id: String, pub status: String, pub outcome: Option<String>,
    pub entry: Option<Value>, pub seq: i64 }
pub struct ApiError { pub code: ErrorCode, pub message: String, pub details: Value }
pub enum ErrorCode { BadRequest, Unauthorized, Forbidden, BadHost, ReadOnly, NotFound,
    UnknownRun, Conflict, TooLarge, UnsupportedMedia, BoardLocked, DbBusy, Unavailable,
    InUse, BackupFailed, GitFailed, Internal }
pub struct Config { .. }
pub fn load_config(paths: &Paths) -> Result<Config, ConfigError>;
pub struct Paths { pub coord_home: PathBuf, pub home: PathBuf, .. }
pub trait Clock: Send + Sync { fn now(&self) -> OffsetDateTime; }
pub struct Bus { .. }
pub fn redactor(extra: &[String]) -> Redactor;
pub fn iso(t: OffsetDateTime) -> String;
```

`Config` reads and writes `config.json` with every key of DESIGN 1 and keeps unknown keys.
`Paths` honours `COORD_HOME`, `DASHBOARD_HOME` and `DASHBOARD_PORT` as the Node server does.
`iso` prints the exact `Date.toISOString` form (`2026-10-26T12:00:00.000Z`). `Bus` holds the
`tokio::sync::broadcast` channels for doc changes, resets, run log lines, applied owner posts,
board exports and notifications. The logger writes JSON lines to `logs/server.log` and rotates
at 5 MB keeping five files, through a `tracing` subscriber with a size-rotating writer.

### argus-store

```rust
pub struct Db { .. }
impl Db {
    pub fn open(file: &Path) -> Result<Db, StoreError>;
    pub fn tx<T>(&self, f: impl FnOnce(&Tx) -> Result<T, StoreError>) -> Result<T, StoreError>;
    pub fn docs(&self) -> DocStore<'_>;
    pub fn kv_get(&self, key: &str) -> Option<String>;
    pub fn kv_set(&self, key: &str, value: &str);
    pub fn next_counter(&self, key: &str) -> i64;
}
impl DocStore<'_> {
    pub fn list(&self, scope: &str, coll: Coll) -> Vec<DocRow>;
    pub fn get(&self, scope: &str, coll: Coll, id: &str) -> Option<DocRow>;
    pub fn query(&self, scope: &str, coll: Coll, q: &Query) -> Vec<DocRow>;
    pub fn apply(&self, scope: &str, changes: Vec<DocChange>) -> Applied;
    pub fn replace(&self, scope: &str, colls: &[Coll], all: Vec<DocChange>) -> Applied;
    pub fn clear(&self, scope: &str);
    pub fn changes_since(&self, scope: &str, seq: i64) -> Option<Vec<AppliedChange>>;
    pub fn seq(&self) -> i64;
}
pub struct ActivityLog { .. }
impl ActivityLog {
    pub fn new(db: Arc<Db>, clock: Arc<dyn Clock>, log: Arc<Logger>) -> ActivityLog;
    pub fn add(&self, new: NewActivity) -> Option<ActivityEntry>;
    pub fn list(&self, filter: &ActivityFilter) -> Result<Vec<ActivityEntry>, StoreError>;
    pub fn mirror(&self);
    pub fn note(&self, note: &LogNote<'_>);
}
```

One writer connection lives on a dedicated thread behind a channel; readers use a small pool of
read-only connections (WAL lets them read while the writer works). `apply` and `replace` emit
one bus event per non-empty result. `ActivityLog` is DESIGN 13.11: `add` never fails its caller,
`list` filters, the last 1000 rows stay, the newest 300 are `activity` docs of `_portfolio`
(`mirror` writes them again after a hard resync), `note` is the Node `wrapLog`: a service's warn
or error line, one per message within a minute.

### argus-board

The board file belongs to the coord CLI. Argus reads every board and writes one only for an owner
post, a proof, an agent close, a reconcile and the worktree keep cleanup, under the same lock.

```rust
pub struct Board(Value);
pub fn read_board(path: &Path) -> Result<Board, BoardError>;
pub fn normalize(raw: Value) -> Board;
pub fn try_lock(dir: &Path) -> Option<BoardLock>;
pub async fn with_board<T>(dir: &Path, wait: Duration, f: impl FnOnce(&mut Board) -> T) -> Result<T, BoardError>;
pub fn save_board(dir: &Path, board: &Board) -> Result<(), BoardError>;
pub fn apply_entry(board: &mut Board, entry: Value, known: &dyn Fn(&str) -> bool) -> ImportResult;
pub fn apply_portfolio_entry(p: &mut Board, entry: Value) -> ImportResult;
pub fn export_docs(board: &Board, proof_url: &dyn Fn(&Value) -> String) -> Vec<DocChange>;
pub fn summarize(key: &str, board: &Board) -> Value;
pub trait Boards: Send + Sync { fn board(&self, key: &str) -> Option<Arc<Board>>; fn keys(&self) -> Vec<String>; }
```

The lock protocol is the CLI's, bit for bit: `board.lock` created with create-new semantics, a
lock older than 15 s removed as stale, waits polled from 25 ms doubling to 200 ms, a `board_locked`
error after the wait. `save_board` writes `board.json.<pid>.tmp` as two-space JSON plus a newline,
then renames, retrying up to 8 times 25 ms apart on a sharing violation, and removes the temp file
on final failure. A board whose `schema` is newer than the one Argus knows is read but never
written. `Board` keeps the parsed `serde_json::Value` with `preserve_order`, so a save keeps every
field Argus does not know, in its order.

`apply_entry` is the port of `importEntries` and the `APPLY` table of `verbs/sync.mjs` and
`ops.mjs`. `export_docs` is `exportDocs` and `publicDoc`. `summarize` is the portfolio card. The
`limits.mjs` table and `OWNER_TEXT_MAX` (50000) live here too. The same crate later serves
`argus-coord`, so the board semantics exist once in Rust.

### argus-sync

The Node `src/sync`, DESIGN 4. It implements `Boards`, owns one async queue per scope (every
mutation of a scope runs in arrival order), and exposes:

```rust
pub struct SyncService { .. }
impl SyncService {
    pub async fn start(&self) -> Result<(), SyncError>;
    pub fn projects(&self) -> Vec<ProjectRow>;
    pub fn run_defs(&self, key: &str) -> Vec<RunDef>;
    pub async fn post_entry(&self, scope: &str, body: Value, source: Source, via: Option<String>) -> Result<PostResult, ApiError>;
    pub async fn send_to_lead(&self, project: &str, text: &str, decision: bool, via: &str) -> Result<PostResult, ApiError>;
    pub async fn attach_proof(&self, key: &str, test: &str, src: ProofSource, caption: &str, by: &str) -> Result<Proof, ApiError>;
    pub async fn resync(&self, project: Option<&str>, hard: bool) -> Result<Resynced, ApiError>;
    pub fn cards(&self) -> Vec<Value>;
    pub fn health(&self) -> WatcherHealth;
}
```

### argus-server

`axum` routes for every path of the HTTP section below, the security layer, the four SSE
streams, page serving, `/files`, and the composition root:

```rust
pub struct Services { pub config, pub paths, pub db: Arc<Db>, pub bus: Bus, pub log: Arc<Logger>,
    pub activity: Arc<ActivityLog>, pub tray: TrayFiles, pub sync: Option<Arc<SyncService>>,
    pub runs: Option<Arc<RunService<SystemHost>>>, pub launcher_stale: Option<Arc<LauncherStale>>,
    pub wake: Option<Arc<Mutex<WakeService>>>, pub agent_watcher: Option<Arc<AgentWatcher>>,
    pub agent_reaper: Option<Arc<AgentReaper>>, pub agent_reconciler: Option<Arc<AgentReconciler>>,
    pub procs: Option<Arc<ProcsService>>, pub worktrees: Option<Arc<WorktreeService>>,
    pub notify: Option<Arc<Notify>>, pub notifier: Option<Arc<dyn Notifier>>, .. }
pub async fn serve(opts: ServeOptions) -> Result<ServerHandle, ServeError>;
```

A service that is off (an option, a failed build) is `None` and its routes answer 503
`unavailable`. `activity` is never `None`: every service writes to it.

#### Service layout

One convention for every service, so a service is wired by touching only its own files:

| File | Holds |
|---|---|
| `services/<svc>.rs` | `build(&Services, &ServeOptions) -> Option<Arc<T>>`, `start(&Arc<Services>)`, `stop(&Services)`, and the service's test seams as a `<Svc>Options` struct on `ServeOptions` |
| `adapters/<svc>.rs` | the crate's store, host and sink traits implemented over `DocStore`, the tables, `BoardReader`, the wake engine and `ActivityAdapter` |
| `adapters/activity/<crate>.rs` | the crate's activity port over `ActivityLog` (already written for every crate) |
| `routes/<svc>.rs` | `register(&mut Router)`, called by one line of `routes::register_all` |
| `tests/it/<svc>.rs` | the Node HTTP test of the service, ported, over `TestServer` |

`services/compose.rs` holds the three orders and already calls every service: `build_all`
(runs, watcher, sync, reaper, reconciler, wake, stale, procs, worktrees, cache, notify), `start_all` (the
Node boot order: wake before sync, whose boot recovery may wake threads) and `stop_all` (the
reverse). A service not wired yet has a stub of those functions answering `None` or doing nothing;
wiring it replaces the bodies, never `compose.rs`. Rules of an adapter: a trait method that fails
returns the crate's error as text, nothing blocks the runtime (a blocking service call from a route
goes through `spawn_blocking`), a host method for a service that is `None` is a no-op.

The run service, the launcher wakes and the worktree scanner reach the wake engine through
`Services::wake_link`, a `WakeBridge` whose calls do nothing until the wake wiring plugs the
engine in with `set`. A service's timers are kept in `Services::tasks`, by service name, and
ended at stop.

The activity log is the one service built before `Services` exists. `services/procs.rs` with
`adapters/procs.rs` and `routes/procs.rs` is the worked example.

Test servers keep the services that sample or act on the machine off. `TestOptions::tweak` changes
the `ServeOptions` last: the process monitor test turns `procs` on with a fake sampler and killer.

### argus-wake, argus-procs, argus-worktrees, argus-launchers, argus-agents

Each is described under Services below. Each exposes one service struct with `start`, `stop`
and the calls its routes need, takes a `Clock` and the `Bus`, and writes its docs through the
`DocStore`. Timers use `tokio::time`, so tests drive them with a paused clock.

### argus-notify, argus-native, argus-mcp

Described under Services below.

### argus-testkit

`start_test_server(seed)` returns the base URL, the port, the temp `COORD_HOME`, a JSON client
and a stop handle. `make_home` writes the fixture boards `alpha`, `beta` and `_portfolio` of
DESIGN 9 with the run defs `hello`, `fail`, `slow` and `app`. The fixture boards are JSON files
generated once from the Node `test/fixtures/make-home.mjs` and committed, so both servers test
the same data. It also carries an SSE frame reader, a fake clock, throwaway git repositories and
a locator for the Node coord CLI (`ARGUS_COORD_CLI`).

## Key libraries

| Library | Used in | Reason |
|---|---|---|
| `tokio` | all services | Async runtime, child processes, timers that tests can pause |
| `axum` 0.8 | server | Router, extractors and SSE bodies on `hyper`, the common choice |
| `tower` | server | The loopback, Host, Origin and content-type checks as one layer |
| `serde`, `serde_json` with `preserve_order` | all | JSON with insertion order, as `JSON.stringify` writes it |
| `rusqlite` with `bundled` | store | SQLite with no system library, same file format as `node:sqlite` |
| `notify` | sync, agents | Recursive file watching on `COORD_HOME` and transcript folders |
| `sha1`, `sha2` | store, worktrees, sync | Doc hashes, worktree ids, attachment digests |
| `time` | core | ISO timestamps with milliseconds, epoch math |
| `regex` | core, board | Validation and redaction patterns |
| `fancy-regex` | procs | Process rules use look-behind (`(?<![.])`), which `regex` lacks |
| `clap` | argus | The subcommands and flags of the `dashboard` bin |
| `tracing`, `tracing-subscriber` | core | Structured JSON-lines server log |
| `rust-embed` | server | Web build embedded in release, read from disk in debug |
| `mime_guess` | server | Content types for static and file routes |
| `reqwest` (rustls, no default features) | argus, mcp | Stdio bridge forwarding, wake-stream client |
| `ureq` (no TLS, no default features) | coord | Blocking CLI calls to the loopback server, no runtime to start on the hook path |
| `getrandom`, `subtle` | core, server | The config token and its constant-time compare |
| `base64` | mcp | `attach_proof` payloads |
| `thiserror` | libraries | One error enum per crate; `anyhow` only in the binary |
| `sysinfo` | native | Process list, parents, memory, CPU time, start time, command lines |
| `windows` | native, launchers | Hung-window check, WinRT toasts, named mutex, process creation flags |
| `tray-icon`, `muda` | native | Tray icon and a native menu that follows the Windows 11 light and dark theme |
| `tempfile` | testkit | Temp homes and folders in tests |

Git stays the `git` executable behind the allowlist doors of DESIGN 13.12, so Argus runs the very
commands the Node scanner runs and stays read-only by construction.

## Data model

### Data folders and config

Argus uses the Node paths: `COORD_HOME` (default `~/.coord`) holds the boards, `DASHBOARD_HOME`
(default `<COORD_HOME>/dashboard`) holds `config.json`, `dashboard.db`, `server.pid`, `logs/`,
`attachments/`, `process-rules.json`, `worktree-backups/`, `tray.json` and `quit.flag`. Every
key of `config.json` keeps its meaning, `notify` (DESIGN 13.13) included. Argus adds one key,
`exe`: the absolute path of `argus.exe`, rewritten on every start.

`installDir` keeps pointing at a folder with `bin/dashboard.mjs` (the shim of the switch-over), so
the coord CLI that spawns `node <installDir>/bin/dashboard.mjs start` keeps working unchanged.

### SQLite schema

`crates/argus-store/migrations/0001.sql` to `0006.sql` are the six Node migrations of
`src/migrations.mjs`, copied statement for statement. `PRAGMA user_version` counts applied
migrations, as in Node, so Argus opens the Node `dashboard.db` as it is and applies only what is
newer. Pragmas on every open: `journal_mode=WAL`, `synchronous=NORMAL`, `foreign_keys=ON`,
`busy_timeout=5000`, `temp_store=MEMORY`. Writes run in `BEGIN IMMEDIATE`. A database newer than
the code refuses to open with the Node message.

| Migration | Tables |
|---|---|
| 1 | `kv`, `docs`, `changes`, `posts`, `launch_defs`, `runs`, `run_logs`, `attachments` |
| 2 | `wakes` |
| 3 | `runs.stdin` |
| 4 | `wake_faults` with one open row per scope and thread |
| 5 | `activity` |
| 6 | `notifications`, the last 50 pings and what became of their toast |

Argus adds no migration of its own before the switch-over. A migration Node adds in the meantime
is copied under the same number, so both servers keep one schema sequence and either can open the
other's database. Argus state that has no Node table (the palette of a board) lives in `kv`
under a prefixed key (`palette:<key>`).

### Docs and the change feed

`docs` is the page-facing mirror: one row per (scope, collection, id), its JSON, the sha1 of the
JSON text, and the seq of its last change. Collections are the 20 of the Node `db.mjs`
(`threads items tests entries locks actions agents log meta runs launches cards builds launchers
wakes registry worktrees procs activity notifications`). `meta` holds `board`, and the
foreign metas `procs` (on `_portfolio`) and `worktrees` (on a project) that the exporter keeps.
`changes` keeps the last 20000 rows and 7 days; a resume past that falls back to a snapshot.

The hash only decides whether a set is a change. Argus builds objects in its own key order, so its
hashes differ from the Node ones for some docs. On the first start against a Node database Argus
runs a hard resync: every scope is cleared, exported again and sent `reset`. Clients reconnect and
take a snapshot. Content parity (same fields, same values) is what the tests check, never the
hash text.

### Board ingestion

The exporter is DESIGN 4.2:

1. A `notify` watcher on `COORD_HOME` schedules `<key>` on a change to `<key>/board.json`, with a
   120 ms trailing debounce and a 1 s maximum wait; temp files, `board.lock`, backups and the
   `dashboard` folder are ignored. A 10 s poll of every board's mtime and size catches what the
   watcher missed and new projects.
2. The read takes no lock (saves are renames). A busy file or bad JSON is retried 3 times 40 ms
   apart, then the last good board stays, the error counts in health, and a retry follows in 2 s.
3. `export_docs` yields every doc of the page collections (`threads items tests entries locks
   actions agents log builds launchers`), `cards` when present, `meta/board` (`project, name,
   updatedAt, counters, handedBuild, handedAt`, plus `palette` from kv) and one `launches`
   doc per run def without env values. The launcher docs get the computed usage fields
   (`joined`, `standalone`, `useEveryMs`, `lastUsedAt`, `stale`) with the Play runs overlaid. `DocStore::replace` sets what changed and deletes what is
   gone, in one transaction with one seq. `launch_defs` is replaced in the same transaction.
4. Any project export schedules the portfolio refresh (500 ms debounce): cards with run counts
   and the master launcher's last run, delivery retries of pending portfolio entries, and the
   `_portfolio` entries and meta.

Argus never writes a board from an export, so the watcher event after an Argus write finds equal
hashes and emits nothing.

### Owner posts

`post_entry` is DESIGN 4.3: validate (256 KB body, the `O-<base36>-<4>` id, the reply and op
kinds, the short `limits.mjs` caps for `cli` and `mcp`, only the 50000-character net for `page`), return
the stored result for a known id, insert a `received` row, apply under the board lock on the
scope queue, export, answer `PostResult` with the export's seq, update the `posts` row. A held
lock past 10 s is 503 `board_locked` and the row stays `received` for boot recovery. Board-level
`conflict`, `refused`, `cancelled` and `duplicate` are 200 answers. Every applied post goes on the
bus as `Applied {scope, entry, status, board, before, source}` for the wake policy. Every board
export goes on the bus as `Exported {key, board}` for the automatic pings and the services that
judge again on export (worktree statuses, registry).

## HTTP API

The listener binds `127.0.0.1` only, port 47615 by default. Every rule of DESIGN 3 and 8 holds:
JSON with `application/json; charset=utf-8`, the error shape
`{"error": {"code", "message", "details"}}`, the code to status table, body limits (JSON 1 MB,
entries 64 KB, images `attach.maxBytes`), the peer and Host checks with 421 `bad_host`, the board
alias hosts (`<key>.localhost`, `<thread>.<key>.localhost`, the reserved `argus.localhost` and
`overlord.localhost`, and the label `argus-app` that names the project `argus`), the write checks on `Origin`, `Sec-Fetch-Site` and `Content-Type`, the
page versus CLI classification of a caller, the optional `requireToken` mode with its cookie and
bearer rules, no CORS headers ever, the CSP and the security headers on pages, and redaction in
the log. JSON field names and value types are copied from the Node answers; key order is free.

### Routes kept from the Node server

| Method | Path | Area |
|---|---|---|
| GET | `/api/p/:scope/db/:coll` | db shim |
| GET | `/api/p/:scope/db/:coll/:id` | db shim |
| POST | `/api/p/:scope/db/:coll/query` | db shim |
| PUT | `/api/p/:scope/db/:coll/:id` | db shim (entries post, else `read_only`) |
| PATCH, DELETE | `/api/p/:scope/db/:coll/:id` | db shim (`read_only`) |
| POST | `/api/p/:scope/db/batch` | db shim |
| GET | `/api/p/:scope/stream` | doc stream (SSE) |
| POST | `/api/p/:key/entries` | owner posts |
| POST | `/api/p/:key/tests/:id/result` | owner posts |
| POST | `/api/p/:key/tests/:id/proofs` | proofs (image body) |
| POST | `/api/portfolio/send` | portfolio posts |
| GET | `/api/p/:key/launches` | runs |
| POST, GET | `/api/p/:key/runs` | runs |
| GET | `/api/runs` | runs across scopes |
| GET | `/api/p/:key/runs/:rid` | runs |
| POST | `/api/p/:key/runs/:rid/stop` | runs |
| POST | `/api/p/:key/runs/:rid/stdin` | runs |
| GET | `/api/p/:key/runs/:rid/logs` | runs |
| GET | `/api/p/:key/runs/:rid/logs/stream` | run log stream (SSE) |
| POST, GET | `/api/p/:key/runs/:rid/attachments` | runs (image body on POST) |
| GET | `/api/p/:key/wakes/stream` | wake stream (SSE) |
| GET, POST | `/api/p/:key/wakes` | wakes |
| GET | `/api/p/:key/wakes/listeners` | wakes |
| GET | `/api/p/:key/wakes/notices` | wakes (the passive notices digest of the prompt hook) |
| GET | `/api/p/:key/presence` | presence stream (SSE) |
| GET | `/api/presence` | presence |
| GET | `/api/wake-faults` | wake faults |
| POST | `/api/wake-faults/clear` | wake faults (CLI only) |
| GET | `/api/procs` | processes |
| POST | `/api/procs/sample` | processes |
| GET, POST | `/api/procs/rules` | processes |
| POST | `/api/procs/rules/remove` | processes |
| POST | `/api/procs/kill` | processes |
| POST | `/api/procs/kill-rule` | processes |
| GET | `/api/activity` | activity |
| GET, POST | `/api/notify` | notifications |
| POST | `/api/notify/test` | notifications (page only) |
| GET, POST | `/api/notify/settings` | notifications (POST page only) |
| POST | `/api/p/:key/worktrees/remove` | worktrees |
| POST | `/api/p/:key/worktrees/prune` | worktrees |
| POST | `/api/p/:key/worktrees/leftovers/remove` | worktrees |
| GET | `/api/agents/reconcile` | agents |
| GET | `/api/health` | admin |
| GET | `/api/version` | admin |
| GET | `/api/projects` | admin |
| POST | `/api/admin/resync` | admin |
| POST | `/api/admin/shutdown` | admin |
| POST | `/api/admin/token/rotate` | admin |
| POST | `/api/auth/code` | admin |
| GET | `/auth` | admin |
| POST, GET, DELETE | `/mcp` | MCP streamable HTTP |
| GET | `/`, `/index.html` | overview page |
| GET | `/p/:key`, `/p/:key/` | board page (redirect, page, 404 page) |
| GET | `/static/*` | icons and legacy static names |
| GET | `/files/:key/proofs/:name` | proof images |
| GET | `/files/:key/a/:attId` | run attachments |

`/api/health` keeps every Node field and adds `impl: "argus"`; `/api/version` keeps `version`,
`schema` and `pages` (the web build id under both `board` and `portfolio`, so a page older than
the server reloads itself) and reports `rust` in place of `node`.

### Routes added by Argus

| Method | Path | Answer |
|---|---|---|
| PUT | `/api/p/:key/palette` | `{palette}` in (`basic brock rotp archipelia bxel`), `{palette}` out, page only |

No other route is added before the switch-over, so the Node and Rust servers answer the same set.

### SSE contract

Argus writes SSE frames by hand, so the bytes match the Node server: field order `id`, `event`,
`data`, each line ended by `\n`, a blank line after each frame, the data as one JSON line, a
`: ping` comment every 15 s, `Cache-Control: no-cache`, `Content-Type: text/event-stream`.

```text
id: 1040
event: snapshot
data: {"scope":"rotp","seq":1040,"coll":"items","docs":[{"id":"I-1"}]}

event: ready
data: {"seq":1040}

id: 1042
event: changes
data: {"scope":"rotp","seq":1042,"changes":[{"coll":"items","id":"I-3","op":"set","data":{}}]}

event: reset
data: {"reason":"resync"}

: ping
```

| Stream | Events | Rules |
|---|---|---|
| doc stream | `snapshot`, `ready`, `changes`, `reset` | `c` filter, resume by `Last-Event-ID` or `since`, snapshot fallback, one `changes` per apply, drop at 1 MB buffered, 32 clients per scope |
| run log stream | `log`, `status` | `after` resume, a first `status`, ends after a final `status` |
| wake stream | `ready`, `wake`, `replaced` | `thread`, `v=2` one listener per thread key, legacy listeners, queued replay within 24 h, 64 per project |
| presence stream | `ready` | one per open board tab with `?thread=`, 8 tabs per thread |

Notifications reach the page as `notifications` docs on the `_portfolio` doc stream, as in Node.

## Services

### Wake engine (argus-wake)

The port of DESIGN 13.3 to 13.9, in modules that mirror the Node files:

| Module | Role |
|---|---|
| `policy` | Pure `wakes_for(applied) -> Vec<Wake>`, every row of the DESIGN 13.3 table, `WAKE_REASONS`, `EXTERNAL_REASONS`, `IMMEDIATE_REASONS` |
| `engine` | Windows per (project, thread), `wake.windowMs` default 15 s, 8 lines then `+N more`, 300-character texts, delivery, queued replay, boot recovery, the `wakes` mirror of the last 50 |
| `store` | SQL on `wakes`, `last_at(scope, thread, reason, subject)` |
| `listeners` | The SSE listener table, v2 replacement, legacy listeners, labels |
| `registry` | One row per live thread, states, the three failures, `wake_faults`, the Overlord's wake, the `registry` docs |
| `presence` | Open board tabs, `board-closed` faults |
| `agent_wakes` | `agent-stale` and `agent-checkin` |
| `item_wakes` | `kanban-stale` reminders and the Overlord's digest |
| `sync_wakes` | `remote-sync-due` wake of a lead a day without `coord remote synced` (docs/remote-sync.md) |
| `wake_policy` | The one table reason to wake or passive, `class_of`, the `wakePolicy` debounce and digest settings |
| `notices` | The digest of the passive notices since a thread's last turn, and its one line |

It listens to `Applied` events from the bus and offers `submit(Wake)` to other crates (procs,
worktrees, agents). It never decides from page data: it reads boards through the `Boards` trait.

#### The wake table

A thread stays idle until Argus signals a reason on the board (docs/wake-policy.md). The engine
applies this table in `submit`, the one place every wake is queued, so every producer obeys it. A
passive row is recorded (status `passive`), mirrored and counted in the board header, and printed
in the next prompt hook's digest line; it never reaches a listener and is never replayed.

| Reason | Class | Why |
|---|---|---|
| `owner-answer`, `owner-note`, `owner-decision`, `item-assigned`, `resume` | wake | the owner posted for this thread |
| `portfolio-decision`, `portfolio-note` | wake | the Overlord or the owner sent it down |
| `lead-message` | wake | a lead's decision: it asks for action |
| `thread-message` | wake | a message to this thread (also a worktree request it filed, done or refused) |
| `agent-idle`, `agent-orphan` | wake | one of its agents finished or is gone |
| `test-fail`, `test-partial`, `test-blocked`, `bundle-complete`, `run-failed` | wake | its own test or run result |
| `process-reaped` | wake | the reaper stopped its own hung run |
| `lead-info` | passive | a lead's note, for information only |
| `wake-fault` | passive | `board-closed`, `no-listener`, `no-reaction`, `not-listening`, `delivery-error` of a thread |
| `agent-stale`, `agent-checkin` | passive | a quiet agent reminder |
| `kanban-stale` | passive | item staleness |
| `process-alert` | passive | hogs and orphans the process monitor saw |
| `worktree-stale`, `worktree-review` | passive | worktree hygiene |
| `launcher-stale`, `launcher-master-missing` | passive | launcher hygiene |
| `remote-sync-due` | passive | the daily remote sync reminder |

A reason the table does not list wakes. On top of the table, `wakePolicy.debounceMs` (5 min)
lets a thread be woken at most once per window; a wake inside it joins one digest sent
`wakePolicy.digestMs` (20 min) after the last wake, and an owner reason goes at the end of the
window instead.

### Process monitor (argus-procs)

DESIGN 13.10. The rules file `process-rules.json` with `DEFAULT_RULES`, reload on change, a bad
edit keeps the previous rules, regexes on slash-normalised text compiled with `fancy-regex` and
case-insensitive, first rule wins, the protected list, the status flags in their order, the docs
`procs` and `meta/procs` of `_portfolio`, alerts after two samples coalesced into one
`process-alert` wake, kills with the reused-pid guard. Sampling sits behind a trait:

```rust
pub trait Sampler: Send + Sync { fn sample(&self) -> Result<Vec<ProcSample>, SampleError>; }
```

Slice 4 ships the PowerShell sampler of the Node server (one hidden `powershell.exe
-EncodedCommand` printing `Win32_Process` as JSON, the `ps` parser elsewhere) so the numbers match
Node while the tab is ported. Slice 6 swaps in the native sampler of `argus-native`. Kills call
`taskkill /PID n [/T] /F` as Node does.

A thread may end its own project's processes (`coord proc kill` from a project's thread), and the
reaper ends hung searches and test or build runs after each sample. Both go through one scope
rule: see [process-reaper.md](process-reaper.md).

### Worktrees (argus-worktrees)

DESIGN 13.12 module by module: `git` (the read-only allowlist door, 2 processes at most, 10 s
each), `scan` (rule expansion, the per-folder facts, live check every 20 s), `status` (the ten
statuses, judged again on each board export), `leftover` (bounded measuring, 20000 entries and
4 s per folder, junctions never followed), `remove` (the guard tables in order, the write door,
backups with bundle, patch and untracked copy, deletion jobs `L-n` with the rename first),
`service` (docs, `meta/worktrees`, the `worktree-stale` wake, the sent map in kv). The in-use
guard asks `argus-procs` for its last sample. The current Node rules hold: a rule path is a
literal parent folder with `include` and `exclude` name globs (a stored wildcard rule is invalid
and skipped), a worktree is judged merged against the remote or the local default branch, a
clean worktree whose HEAD a ref keeps is removable with no approval, the branch decision is
separate (ancestry, `git cherry`, or a merged pull request through an optional `gh`), links
inside a worktree are removed as links before `git worktree remove` on Windows, a folder git
could not delete is finished by Argus once git has let go of it, and a thread may delete
leftovers younger than one day only with a reason and never under six hours.

### Runs and launchers (argus-launchers)

DESIGN 5, 6 and 13.7. Defs come only from `board.runDefs`, read through `Boards`. Ids `R-<n>` per
scope from `next_counter`. The lifecycle `queued`, `running`, `exited`, `failed`, `stopped`, the
`parallel` limit with `already`, `maxConcurrent` queueing, timeouts, the stdin pipe for launch and
bundle defs, the colour environment for hidden bundle runs, log chunks of 16 KB batched every
250 ms, the head and tail cap, redaction, `lastLine`, retention, the `runs` docs at most four per
second for progress, boot recovery of orphans, and shutdown that stops hidden runs and leaves
visible ones.

Spawning uses `tokio::process::Command` with `cmd.exe /d /s /c <cmd>`. A hidden run gets
`CREATE_NO_WINDOW`. A visible run gets `CREATE_NEW_CONSOLE`. Stop is `taskkill /PID <pid> /T`,
then `/F` after `stopGraceMs`, as in Node. The launcher rules (`launcher-<name>` defs, `master`,
retire, bundle join, `useEvery`) are board semantics and live in `argus-board`; this crate runs
what the defs say. Its `launcher_stale` module is the Node `src/launchers/stale.mjs`: every
10 minutes it sends `launcher-master-missing` to a lead whose board has no live master and
`launcher-stale` to the thread that made a standalone launcher unused past its window, and it
keeps the last Play time per launcher in kv (`launcher-played:<scope>:<name>`).

### Agents (argus-agents)

Three services and one shared reader over one data source (the sub-agent transcripts the session host keeps under
`~/.claude/projects/<slug>/<sessionId>/subagents/`, and the task outputs under the temp folder):

| Service | Node source | Role |
|---|---|---|
| `watcher` | `src/agents/watcher.mjs` | Tails transcripts once a second, live fields on the `agents` docs, never written to the board |
| `reaper` | `src/agents/reaper.mjs` | Closes a hook record whose transcript, task output and worktree stayed quiet for `agents.reapMs` (`agents.reapOpusMs` for opus-level agents) |
| `reconciler` | `src/agents/reconciler.mjs` | Verify ids, link, duplicates, classify, resume (also a record the reaper closed while the agent still worked); the report route |
| `evidence` | `src/agents/evidence.mjs` | What the assistant keeps for a session (transcript, meta, task output) and the newest sign of life of a record; shared by the reaper and the reconciler |

The folders come from `agents.claudeHome` and `agents.claudeTemp` in config, so tests point them
at temp folders.

### Activity log

`ActivityLog` in `argus-store`, DESIGN 13.11. Every service writes its one-liners through it, as
`NewActivity` (kind, level, text, project, thread). `ActivityLog::note` turns a service's warn and
error log lines into rows at most once a minute per message. `GET /api/activity[?limit&kind&level]`
answers `{activity}` newest first. The server writes `dashboard started` and `dashboard stopped`,
and writes the docs again when a reset of `_portfolio` reaches the bus.

Each service crate defines its own activity trait, and `ActivityAdapter` (`adapters/activity/`)
implements it: `argus-wake`, `argus-notify` (kind `note`) and `argus-agents` as sinks; the process
monitor, the worktree service and the launcher wakes as methods (`procs_line`, `worktree_line`,
`stale_note`) their own adapters call. The run service's warnings go to the server log only.

### Notifications (argus-notify)

The port of DESIGN 13.13. A thread, the Overlord or Argus pings the owner when he is needed;
every ping is recorded and, when its level says so, shown as a Windows toast.

| Part | Rule |
|---|---|
| Levels | `info` recorded only (toast with `notify.toastInfo`), `ping` recorded and toasted, `urgent` toasted as a reminder that stays until dismissed; sound per level from `notify.sound` |
| `send` | `{title, text, level, project, thread, by, url, source}` in, `{recorded, id, n, level, toast, why, notification}` out; never fails its caller; plain text, title 60 and text 240 characters, control and bidi marks removed |
| Record | table `notifications`, the newest 50 kept; docs `n-<n>` on `_portfolio`; `toast` is `shown`, `skipped`, `failed` or `limited` with a `why` sentence |
| Limits | per project and thread 3 a minute and 20 an hour, 8 a minute overall; over a limit the ping is recorded `limited`; one summary toast a minute per thread |
| Automatic pings | on each board export: a handed build (`ready to test`) and a new open question by a thread; markers in kv `notify:seen` so a restart never tells twice |
| Activity | one `note` line per `ping` or `urgent`, `warn` when the toast failed |
| Config | `notify.os`, `notify.auto.handed`, `notify.auto.ask`, `notify.quietLevels`, `notify.sound`, `notify.toastInfo`, read on every send, edited from the Notifications tab |

The toast XML is the Node `toastXml` output byte for byte (protocol activation with the board
URL, a tag of 12 hex characters per project and thread, group `argus`, every value escaped). The
sender is a trait so slice 4 can ship the Node sender (one hidden `powershell.exe` with the XML in
the environment) and slice 6 the native one. A failed toast falls back to the tray balloon, as in
Node.

### Native helper (argus-native)

Windows only; every function has a non-Windows build that returns `Unsupported`, so the workspace
builds and tests on any platform.

| Piece | Design |
|---|---|
| Toasts (slice 6, first) | WinRT `ToastNotificationManager` in-process through the `windows` crate, fed the same XML, tag and group as the Node sender. Protocol activation opens the board URL in the browser, so no COM activator is needed. The notifier keeps the app id Node uses (the Windows PowerShell id), so no registration is written; an Argus app id of its own needs a Start menu shortcut and is an owner question below. |
| Tray (slice 6, second) | A separate `argus tray` process, single instance through a named mutex per dashboard home, `tray.json` and `quit.flag` as DESIGN 7.6, and the balloon fallback of 13.13. `tray-icon` draws the Argus icon for running and stopped; `muda` builds the menu with the Windows 11 look and the system light or dark theme. Menu: Open Argus (also left click), status line with live runs, Notifications (opens `#notifications`), Open log folder, Restart server, Start server when stopped, Quit Argus. It polls `/api/health` every 5 s. Every command it runs is a hidden process. |
| Process sampling (slice 6, third) | `sysinfo` lists pid, parent, name, command line, executable, working set, start time and CPU time in-process, with no PowerShell child. `Responding` comes from `EnumWindows` plus `IsHungAppWindow` for processes that own a top-level window. The sampler's own process and its children are left out, as in Node. |

The tray starts the server with `argus start`, which spawns `argus serve` detached and hidden
(`DETACHED_PROCESS`, `CREATE_NO_WINDOW`, `CREATE_BREAKAWAY_FROM_JOB`) and waits for health. The
PowerShell workarounds of DESIGN 7.6 are gone because Argus controls its own creation flags.

### MCP (argus-mcp)

DESIGN 7.1. The server name stays `coord-dashboard`, so every rendered `.mcp.json` and every
permission rule `mcp__coord-dashboard__<tool>` stays valid. The tool host is written by hand over
`serde_json` (JSON-RPC 2.0, revision `2025-06-18`, stateless, no session id, notifications 202,
`GET /mcp` 405), because the Node transport is that small and its exact behaviour is the contract.
`TOOL_DEFS` is a JSON file in the crate, generated once from the Node `tools/list` answer and
compared with it in a test. Tools: the 23 of DESIGN 7.1, 13.7 and 13.13 (14 read tools, the 8
tools that act as the owner, and `notify_owner`, which speaks to him).
Resources and templates as listed there. The permission model (`mcp.allow`, `deny`, `readOnly`)
is unchanged.

`argus mcp [--project k]` is the stdio bridge: newline-delimited JSON-RPC, `initialize` and
`tools/list` answered locally, everything else forwarded to `/mcp` with the default project
header, the server started with the tray when it is down, the `quit.flag` rule, logs on stderr
only.

### The argus binary

| Subcommand | Node equivalent |
|---|---|
| `argus serve [--port n] [--data-dir dir] [--shadow]` | `dashboard serve` |
| `argus start [--foreground]`, `stop`, `status [--json]` | same verbs |
| `argus open [<key>] [--print]` | same verb |
| `argus logs [--lines n] [--follow] [--run R-n --project k]` | same verb |
| `argus resync [--project k] [--hard]`, `token --rotate` | same verbs |
| `argus wake-stream --project k [--thread id] [--id label]` | same verb, same stdout lines and exit codes |
| `argus mcp [--project k]` | same verb |
| `argus tray [run or start or stop or status]` | same verb; `run` is the tray process, `start` launches it detached |
| `argus autostart install or remove or status`, `install-mcp ...` | same verbs |
| `argus parity --left url --right url` | new: the shadow comparison below |
| `argus dns status`, `install --print`, `remove --print` | new: the local names, see Local names |
| `argus coord ...` | slice 8: the coord CLI |

`--shadow` starts Argus with every outward effect off: no board writes (posts answer
`read_only`), no wakes delivered, no process kills, no worktree removal, no toasts (pings are
recorded `skipped`). It is how
Argus runs beside the Node server on real data. `--home` is an alias of `--data-dir`.

## Local names

Off by default; the owner steps are in `docs/local-names.md`. With `localNames.enabled` in
`config.json`, the server binds two sockets on a loopback address of its own (default
`127.65.82.71`, never `127.0.0.1`, so ports 53 and 80 on it cannot conflict with other software):
the `argus-dns` responder on `<address>:53` and a front door on `<address>:80`, which serves the
same app as the main listener. `<key>.<suffix>` and `<thread>.<key>.<suffix>` are board aliases
like the `.localhost` ones: `security::host_alias` accepts both, and the Host and Origin checks
use the door's port (none in the header when it is 80). When port 80 cannot be bound the door
takes the main port on the same address and the names carry it. `argus-core::names` holds the
setting (`LocalNames`) and the door (`Door`) that the server, `argus-notify` (toast links) and the
`argus` binary (`argus open`, `argus dns status`) share. The server reports the live names in the
`localNames` object of the health answer. Argus never runs the NRPT rule that sends `.<suffix>`
to the responder: `argus dns install --print` prints it for the owner.

## Web app

`apps/web`: Vite, React 19, TypeScript, `react-router` 7, vitest with Testing Library and jsdom,
and `@drizztdourden08/tessera`. No other UI or state library.

### Routing

| URL | Screen |
|---|---|
| `/` | Overview: tabs Dashboards, Processes, Activity, Notifications (hash `#dashboards`, `#processes`, `#activity`, `#notifications`) |
| `/p/:key/` | Board: tabs Agents, Activity, Kanban, Tests, Builds, Worktrees (hash per tab, as today) |
| `/p/:key/?thread=<id>` | Board with the presence claim of that thread |

The server answers both URLs with the same `index.html` of the web build, after its own checks
(redirect `/p/:key` to the slash form, 404 page for an unknown key, the alias redirects). The
client router picks the screen. Tabs keep the hash so old links and the board shortcuts land on
the same tab.

### Data layer

`apps/web/src/data` holds the client side of the contract and nothing else:

| Module | Role |
|---|---|
| `api` | `fetch` wrapper for `/api/*` with the error shape as a typed error |
| `scope-stream` | One `EventSource` per scope on `/api/p/:scope/stream`, the shim rules of DESIGN 3.2: state per collection, resume, `reset`, reopen with the union when a collection is added, the error after a 401 or three failed reconnects |
| `use-collection`, `use-doc` | Hooks over `useSyncExternalStore` returning typed docs of a collection or one doc |
| `posts` | `postEntry`, `recordResult`, `attachProof`, `sendDown`, each returning the `PostResult` |
| `runs` | `startRun`, `stopRun`, `tailRun` (log SSE), `sendStdin`, `attachToRun`, with an optional scope for the overview's master Play |
| `presence` | Holds the presence stream while a board is open with `?thread=` |
| `notifications` | `GET /api/notify`, the settings read and write, the test ping |
| `types` | The doc shapes of every collection, written from the Node contract |

A post never updates the page by itself: the reply arrives after the export, and the `changes`
event that carries the new state is already on the stream (DESIGN 3.5).

### Palettes

Tessera switches palette with CSS only: `data-palette="brock"`, `"rotp"`, `"archipelia"` or
`"bxel"` on any element swaps the tokens under it live, and no attribute means the basic palette.

| Place | Palette |
|---|---|
| Overview page root | none (basic): Argus wears the basic palette |
| Overview project card | the card element carries its board's palette |
| Board page root | `document.documentElement` carries the board's palette, set from `meta/board.palette` |

The palette of a board is an Argus setting, stored in kv (`palette:<key>`), exported on
`meta/board` and on the card, and changed from a Select in the board header
(`PUT /api/p/:key/palette`). With no setting, a built-in map applies: `rotp` to `rotp`, `brock` to
`brock`, `archipelia` to `archipelia`, `pixelarttool` to `bxel`, any other key to basic. Switching
board through the header switcher is a client route change, so the attribute changes in the same
render and the page follows at once.

### Tessera pieces per screen

| Screen | Tessera pieces |
|---|---|
| Shell, both pages | `TesseraProvider`, `Tabs` with `Badge`, `ContentHeader`, `Select` (switcher, palette), `Callout` (status notice), `StatusBar`, `Toast` and `ToastStack` |
| Registered strip | `Tag`, `Status`, `Spinner`, `Tooltip` |
| Overview, Dashboards | `Grid`, `ItemCard`, `StatTile`, `Tag`, `Button` (master Play), `Dialog` (send down) with the Markdown editor |
| Overview, Processes | `DataTable`, `FilterBar`, `SegmentedControl`, `Tag` (flags), `ArmedIconButton` (kill), `StackedBar` (memory by rule), `Drawer` (rules), `Callout` (sample error) |
| Overview, Activity | `LogPanel` (kinds and levels as tones, search, filters) |
| Overview, Notifications | `ItemList`, `ListItemRow`, `Tag` (level, project), `FilterBar`, `Toggle` (toast and sound per level), `Button` (test ping), `EmptyState` |
| Board, Agents | `DataTable`, `Tag` with `Tooltip` (model and why), `Status`, `Drawer` (run log) with the terminal view |
| Board, Activity | `ItemList`, `SegmentedControl` (All, Needs you, Log), `Markdown`, the Markdown editor, `Dialog` |
| Board, Kanban | Kanban, `FilterBar`, `SearchInput`, `Toggle` (show done), `Tag` (quiet chip) |
| Board, Tests | `GroupTree` or `DataTable` grouped by build, `Disclosure`, `SegmentedControl` (outcome), `DropZone` (proof paste), `MediaViewer` (proof lightbox), the Markdown editor |
| Board, Builds | `ItemList`, `CopyButton`, `CodeBlock`, `Button` (Play, Stop), `Disclosure` (Retired), the terminal view |
| Board, Worktrees | `DataTable`, `FilterBar`, `Tag`, `CopyValue`, `ProgressBar` (deletion job), `Callout` (banner), an inline confirm under a row, the Markdown editor |

### Requests to the Tessera lead

| Piece | State | Need |
|---|---|---|
| Kanban | requested, in progress | Columns, drag between and within columns, keyboard move, counts per column |
| Markdown editor | requested, in progress | Textarea with live preview and a small toolbar; Ctrl+Enter sends, Enter never sends; list continuation |
| Terminal view | done | It is `LogPanel` in Tessera 0.70: `ansi` for colours, `follow` with Jump to bottom, `onInput` for the input row with history, Copy all in the toolbar. Stderr and system lines are kinds with a tone; Clear view is a button in `toolbarExtra` |
| DataTable row detail | to request | A panel under one row for inline confirms and forms (Worktrees, Processes) |
| Live relative time | to check | "running 12s", "4m ago" ticking without a redraw of the table; `DateText` has no relative preset that ticks |

### Serving the web assets

Release builds embed `apps/web/dist` with `rust-embed`; `argus.exe` is the whole app. Hashed files
under `/assets/` get `Cache-Control: public, max-age=31536000, immutable`; `index.html` gets
`no-store` and the page headers (CSP, `nosniff`, `no-referrer`). The CSP drops the CDN sources of
the Node pages, since fonts come from Tessera and drag and drop from the Kanban composite:
`default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; font-src 'self';
img-src 'self' data: blob:; connect-src 'self'; frame-ancestors 'none'; base-uri 'none';
form-action 'self'`.

Debug builds read the same folder from disk at request time (the `rust-embed` debug mode). Day to
day front-end work runs `pnpm dev`: Vite on port 5173 proxies `/api`, `/files`, `/mcp` and
`/static` to a debug `argus serve`. The proxy keeps the browser's `Origin`, so the debug server
accepts the extra origin `http://localhost:5173`. That allowance exists only under the cargo
feature `dev-origin`, which release builds never enable.

## Switch-over

1. Shadow. From slice 1 on, Argus runs as `argus serve --shadow --port 47616 --data-dir
   <COORD_HOME>/argus-shadow` (`--home` is an alias), reading the real `~/.coord` boards. When
   the shadow home has no database it starts as a copy of the Node one (`VACUUM INTO`, opened
   read-only), so runs, wakes, faults and activity history are there too. `config.json` (its port
   changed to 47616, nothing else) and `process-rules.json` are copied from the Node home the
   same way, each only when absent. The
   Node home (`DASHBOARD_HOME`, default `<COORD_HOME>/dashboard`) is only read, never written.
   The owner opens `http://127.0.0.1:47616/` beside the Node page when a slice lands.
2. Compare. `argus parity --left http://127.0.0.1:47615 --right http://127.0.0.1:47616` reads
   every scope's collections, `/api/projects`, `/api/procs`, `/api/activity`, `/api/runs` and the
   MCP `tools/list` from both servers and prints the differences, comparing JSON by value and
   ignoring seq, timestamps of the sample and key order.
3. Gate. Every row of the parity matrix is `done`, the full Rust and web suites pass, the
   switch-over checklist of `docs/parity.md` holds.
4. Cut. `dashboard stop` and `dashboard tray stop` end the Node server and its tray by their pid
   files (never by process name). The `dashboard` shim on PATH and `<installDir>/bin/dashboard.mjs`
   now start `argus.exe` with the same arguments. Argus starts with the Node data home, takes a
   backup copy of `dashboard.db` named `dashboard.db.node-<stamp>` (the first start of a home
   without the `argus.marker` file, before anything writes), runs one hard resync, and takes
   port 47615 from the `port` of the home's `config.json`. `argus tray start` brings the icon back.
5. Rollback. Stop Argus, point the shim back at the Node bin, start Node. The schema is the same,
   so Node opens the database Argus used; the backup copy is there if it does not. Board files are
   the source of truth and Argus never rewrites them on its own, so nothing on the boards is lost.
6. Later (slice 8). `argus coord` reaches parity with the coord CLI, and the `coord` shim on PATH
   calls it. The hooks keep calling `coord ...`, so no hook changes.

## Open questions

| For | Question | Recommended default |
|---|---|---|
| Owner | Where is a board's palette chosen? | An Argus setting picked in the board header, with the built-in key map as default |
| Owner | Toasts show as "Windows PowerShell" today. Should Argus toasts carry their own name and icon, which needs a Start menu shortcut with an Argus app id? | Keep the PowerShell id until the switch-over, then add the shortcut |
| Owner | The Node dashboard still gains features, so every Node change is ported twice. Freeze Node features from slice 5 on and build new ones only in Argus? | Yes, freeze when slice 5 starts; fixes still land in Node |
| Lead | Owner posts need the board ops in Rust (`apply_entry`) from slice 5, while the Node coord CLI keeps its own copy until slice 8. Accept two copies of the ops for that period? | Yes; the ported `sync.posts`, `owner.text` and `entry.context` cases hold both to the same answers |
| Overlord | The shim at `<installDir>/bin/dashboard.mjs` keeps the coord CLI unchanged. Later, should `dashboard-client.mjs` read `config.json` `exe` and drop the shim? | Yes, after slice 8 |
