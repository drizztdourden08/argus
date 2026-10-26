<!-- @layer docs @kind doc -->
# Argus parity

Argus replaces the Node dashboard only when it does everything the Node dashboard does. The proof
is the Node test folder `X:/ai-config/dashboard/test`: 84 test files and 1298 cases at `ai-config`
commit `4dbe260`. Every case is ported to Rust or to the web app, or carries a written reason why
it does not apply. This page is the matrix the owner measures progress with.

## How a test is ported

- Port the case, not the code. A Node case becomes a Rust or vitest case that sets up the same
  situation and checks the same answer. The Node file is read for what it proves, never translated
  line by line.
- Each ported Rust test file opens with a `//!` line naming the Node file it ports, and each
  ported case keeps the Node case's sentence as its name in snake case. A case that does not apply
  is listed in that `//!` header with its reason. Counting cases per Node file is then a grep.
- Server behaviour is tested as Rust integration tests under `crates/<crate>/tests/`, driving the
  real HTTP API of an Argus server started by `argus-testkit` on an ephemeral port against a temp
  `COORD_HOME` and `DASHBOARD_HOME`. Pure logic (wake policy, rule matching, statuses, parsers) is
  tested on its crate's functions with a fake clock (`tokio::time::pause` and the `Clock` trait).
- The fixture boards `alpha`, `beta` and `_portfolio` are JSON files generated once from the Node
  `test/fixtures/make-home.mjs` and kept in `crates/argus-testkit/fixtures/`. Export results of
  the Node server on those boards are kept as golden JSON next to them and compared by value.
- Tests that drive the Node coord CLI as a client (`sync.watch`, `runs.cli`, `notify.http`,
  `worktrees.http` and the like) keep doing so: `argus-testkit` runs `node <ARGUS_COORD_CLI>` with
  the temp homes, so the server is proven against the real client it must serve at switch-over.
  Without `ARGUS_COORD_CLI` those cases print why and return. In slice 8 the same cases run a
  second time against `argus coord`.
- Page behaviour is tested with vitest, Testing Library and jsdom in `apps/web`, the screens fed
  by a fake scope stream with fixture docs. These replace the Node stub-DOM page tests.
- Tests run as the lead orders them: one haiku agent per finished piece on the changed crates and
  files, the full suites once before a merge to main, a release or the switch-over.

## Slices

| Slice | Name | Content | Exit line |
|---|---|---|---|
| 1 | Store, server, overview | core, store, board read side, sync export, server, doc stream, admin routes, CLI lifecycle, web shell and data layer, Dashboards tab, shadow mode | Shadow Argus shows every real board's cards and `argus parity` finds no doc difference |
| 2 | Board read pages | board page shell, palettes, Activity feed, markdown, proof images | Every board renders read-only in its palette beside the Node page |
| 3 | Wake engine | policy, engine, listeners, wake stream, registry, faults, presence, agent and item reminders, Registered strip | Wake suites green; `argus wake-stream` against shadow Argus prints the Node lines |
| 4 | Overlord page tabs | activity log, process monitor, notifications, the Processes, Activity and Notifications tabs | The four overview tabs match the Node page on real data |
| 5 | Board tabs | board writes, owner posts, Kanban, Tests, Builds, Worktrees, Agents, runs, launchers, agents services | Every board tab matches the Node page and every owner action applies |
| 6 | Native helper | toasts, then tray, then process sampling, in that order | No PowerShell child left; tray and toasts work on the owner's machine |
| 7 | MCP and switch-over | MCP tools, transports, install, the `dashboard` shim, the switch-over | Argus serves port 47615 and every client works unchanged |
| 8 | coord CLI port | `argus coord` verbs and hooks behind the `coord` shim | The coord suites pass against `argus coord`; the `coord` shim calls it |

Slice 7 ends with the switch-over because the MCP bridge is the last client that needs Argus. The
coord port comes after it: until then the Node coord CLI keeps writing the boards and talking to
Argus over HTTP.

## Parity matrix by test file

Status is `todo`, `partial` or `done`. Two files test `ai-config` itself and have no Argus port;
their status is `n/a`.

| Node test file | Feature area | Slice | Rust or web location | Status |
|---|---|---|---|---|
| `activity.test.mjs` | Activity log | 4 | `crates/argus-store/tests/activity.rs` | done |
| `agent.model.test.mjs` | coord agent verbs | 8 | `crates/argus-coord/tests/agent_model.rs` | done |
| `agents.export.test.mjs` | Agents live view | 5 | `crates/argus-agents/tests/export.rs` | done |
| `agents.reaper.test.mjs` | Agent reaper | 5 | `crates/argus-agents/tests/reaper.rs`, `crates/argus-server/tests/agents.rs` (through the sync service) | done (10 of 10 at `4dbe260`; the 5 worktree-aware cases of `62e66c7` are in the fixes table) |
| `agents.reconciler.http.test.mjs` | Agent reconciler | 5 | `crates/argus-server/tests/agents_reconcile_http.rs` | done |
| `agents.reconciler.test.mjs` | Agent reconciler | 5 | `crates/argus-agents/tests/reconciler.rs`, `crates/argus-wake/tests/agents.rs` (the wake reasons and the flagged record) | done (34 of 34 at `4dbe260`; the 3 reopen cases of `62e66c7` are in the fixes table) |
| `agents.watcher.test.mjs` | Agents live view | 5 | `crates/argus-agents/tests/watcher.rs` | done |
| `ansi.test.mjs` | ANSI terminal rendering | 5 | `apps/web/src/features/terminal/ansi.test.ts` | n/a (Tessera LogPanel renders ANSI) |
| `board.shortcut.test.mjs` | coord board shortcut | 8 | `crates/argus-coord/tests/board_shortcut.rs` | done |
| `build.verbs.test.mjs` | coord build and launcher verbs | 8 | `crates/argus-coord/tests/build_verbs.rs` | done |
| `coord.store.test.mjs` | Board file locking | 5 | `crates/argus-board/tests/lock.rs` | done |
| `core.cli.test.mjs` | Server CLI and lifecycle | 1 | `crates/argus/tests/cli.rs` | done (start, stop, status, open, logs, resync, token; autostart with slice 7) |
| `core.config.test.mjs` | Config and data folders | 1 | `crates/argus-core/tests/config.rs` | done |
| `core.db.test.mjs` | SQLite store and migrations; Doc store and change feed | 1 | `crates/argus-store/tests/db.rs` | done |
| `core.http.test.mjs` | HTTP security and errors | 1 | `crates/argus-server/tests/http_security.rs` | done |
| `core.names.test.mjs` | Display names | 1 | `crates/argus-sync/tests/names.rs` | done |
| `core.sse.test.mjs` | Doc stream SSE | 1 | `crates/argus-server/tests/doc_stream.rs` | done |
| `entry.context.test.mjs` | coord entry context rules | 8 | `crates/argus-coord/tests/entry_context.rs` | done |
| `guard.gate.test.mjs` | none: tests the guard in `ai-config` | none | stays in `ai-config` | n/a |
| `hook.agents.binding.test.mjs` | coord session hooks | 8 | `crates/argus-coord/tests/hook_agents_binding.rs` | done |
| `hook.agents.claudeid.test.mjs` | coord session hooks | 8 | `crates/argus-coord/tests/hook_agents_id.rs` | done |
| `hook.agents.test.mjs` | coord session hooks | 8 | `crates/argus-coord/tests/hook_agents.rs` | done |
| `hook.autostart.test.mjs` | coord session hooks | 8 | `crates/argus-coord/tests/hook_autostart.rs` | done |
| `hook.guard.test.mjs` | coord session hooks | 8 | `crates/argus-coord/tests/hook_guard.rs` | done |
| `hook.listen.test.mjs` | coord session hooks | 8 | `crates/argus-coord/tests/hook_listen.rs` | done |
| `hook.transcript.test.mjs` | coord session hooks | 8 | `crates/argus-coord/tests/hook_transcript.rs` | done |
| `http.alias.test.mjs` | Board alias hosts | 1 | `crates/argus-server/tests/alias.rs` | done |
| `inbox.wake.test.mjs` | coord lead, inbox and ping | 8 | `crates/argus-coord/tests/inbox_wake.rs` | done |
| `launcher.verbs.test.mjs` | coord build and launcher verbs | 8 | `crates/argus-coord/tests/launcher_verbs.rs` | done |
| `launchers.stale.test.mjs` | Launchers and bundles | 5 | `crates/argus-launchers/tests/launcher_stale.rs` | done |
| `lead.verbs.test.mjs` | coord lead, inbox and ping | 8 | `crates/argus-coord/tests/lead_verbs.rs` | done |
| `lightbox.test.mjs` | Proof images and lightbox | 2 | `apps/web/src/features/proofs/ProofViewer.test.tsx` | n/a (Tessera MediaViewer) |
| `mcp.http.test.mjs` | MCP HTTP transport | 7 | `crates/argus-server/tests/mcp_http.rs` | done |
| `mcp.install.test.mjs` | MCP install and render | 7 | `crates/argus/tests/install.rs` | done |
| `mcp.server.test.mjs` | MCP tool host | 7 | `crates/argus-server/tests/mcp_server.rs` | done |
| `mcp.stdio.test.mjs` | MCP stdio bridge | 7 | `crates/argus/tests/mcp_stdio.rs` | done |
| `mcp.tools.test.mjs` | MCP tool host | 7 | `crates/argus-mcp/tests/tools.rs` | done |
| `md.test.mjs` | Markdown rendering; Markdown editor | 2, 5 | `apps/web/tests/markdown/*.test.tsx`, `apps/web/tests/board/plain-text.test.ts` | doing (all applicable cases ported; 8 `gap:` cases wait for Tessera, see Markdown gaps) |
| `notify.http.test.mjs` | Notifications to the owner | 4 | `crates/argus-server/tests/notify.rs` | done |
| `notify.test.mjs` | Notifications to the owner; Native toasts | 4, 6 | `crates/argus-notify/tests/notify/`, `crates/argus-native/tests/toast_payload.rs`, `crates/argus-native/tests/toast_winrt.rs` | done (the `toastScript` and fake PowerShell cases do not apply: the notifier is native) |
| `owner.text.test.mjs` | Owner text limits | 5 | `crates/argus-server/tests/owner_text.rs` | done |
| `pages.agents.test.mjs` | Agents tab | 5 | `apps/web/src/pages/board/agents/AgentsTab.test.tsx` | done |
| `pages.board.test.mjs` | Board page shell; Board Activity feed; Kanban tab; Tests tab; Builds tab | 2, 5 | `apps/web/tests/board*/` | doing (110 cases matched by title; 3 added; the rest wait on features, see Board page gaps) |
| `pages.portfolio.test.mjs` | Overview Dashboards tab; Overview Processes tab; Overview Notifications tab | 1, 4 | `apps/web/src/pages/overview/*/*.test.tsx` | done |
| `pages.registry.test.mjs` | Registered strip | 3 | `apps/web/src/features/registry/RegisteredStrip.test.tsx` | done |
| `pages.shim.test.mjs` | Web shell and data layer | 1 | `apps/web/src/data/*.test.ts` | done |
| `pages.worktrees.test.mjs` | Worktrees tab | 5 | `apps/web/src/pages/board/worktrees/WorktreesTab.test.tsx` | done |
| `portfolio.longsend.test.mjs` | Portfolio send down | 5 | `crates/argus-server/tests/portfolio_send.rs` | done (the two CLI cases need `ARGUS_COORD_CLI`) |
| `procs.http.test.mjs` | Process alerts and kills | 4 | `crates/argus-server/tests/procs_http.rs` | done |
| `procs.monitor.test.mjs` | Process rules and sampling; Process alerts and kills | 4 | `crates/argus-procs/tests/monitor.rs` | done |
| `procs.sampler.test.mjs` | Process rules and sampling; Native process sampling | 4, 6 | `crates/argus-procs/tests/sampler.rs`, `crates/argus-native/tests/sample.rs` | done |
| `render.mcp.test.mjs` | MCP install and render | 7 | stays in `ai-config`, run there at switch-over | todo |
| `render.publish.test.mjs` | none: tests the render in `ai-config` | none | stays in `ai-config` | n/a |
| `runs.cli.test.mjs` | coord run client | 5, 8 | `crates/argus-server/tests/runs_cli.rs` | done |
| `runs.http.test.mjs` | Runs lifecycle | 5 | `crates/argus-server/tests/runs_http.rs` | done |
| `runs.kill.test.mjs` | Runs lifecycle | 5 | `crates/argus-launchers/tests/it/runs_kill.rs`, `crates/argus-launchers/tests/it/system_host.rs` (the stubborn process) | done |
| `runs.lifecycle.test.mjs` | Runs lifecycle | 5 | `crates/argus-launchers/tests/lifecycle.rs` | done |
| `runs.logs.test.mjs` | Run logs and terminal | 5 | `crates/argus-launchers/tests/logs.rs` | done |
| `runs.recover.test.mjs` | Runs lifecycle | 5 | `crates/argus-launchers/tests/recover.rs` | done |
| `runs.server.test.mjs` | Runs lifecycle | 5 | `crates/argus-server/tests/runs_server.rs` | done |
| `runs.terminal.test.mjs` | Run logs and terminal | 5 | `crates/argus-launchers/tests/it/runs_terminal.rs`, `crates/argus-server/tests/runs_stdin.rs` (the stdin route) | done |
| `sync.export.test.mjs` | Board ingestion and export; Portfolio cards | 1 | `crates/argus-sync/tests/export.rs` | done |
| `sync.posts.test.mjs` | Owner posts; Test results and proofs | 5 | `crates/argus-server/tests/posts.rs` | done |
| `sync.watch.test.mjs` | Board ingestion and export | 1 | `crates/argus-sync/tests/watch.rs`, `crates/argus-sync/tests/watch_cli.rs` (Node CLI and `argus coord`) | done |
| `tray.test.mjs` | Tray icon and menu | 6 | unit tests in `crates/argus-native/src/tray/`, `crates/argus/tests/tray_cli.rs` | done (no case starts the tray: it would put an icon on the owner desktop; the PowerShell script cases do not apply) |
| `wake.agents.test.mjs` | Agent wakes | 3 | `crates/argus-wake/tests/agent_wakes.rs` | done |
| `wake.cli.test.mjs` | Wake stream command | 3 | `crates/argus/tests/wake_stream.rs` | done |
| `wake.engine.test.mjs` | Wake engine and listeners | 3 | `crates/argus-wake/tests/engine.rs` | done |
| `wake.faults.test.mjs` | Wake registry and faults | 3 | `crates/argus-server/tests/wake_faults.rs` | done |
| `wake.http.test.mjs` | Wake engine and listeners | 3, 5 | `crates/argus-server/tests/wake_http.rs` | done |
| `wake.items.test.mjs` | Kanban stale reminders | 3 | `crates/argus-wake/tests/item_wakes.rs` | done |
| `wake.policy.test.mjs` | Wake policy | 3 | `crates/argus-wake/tests/policy.rs` | done |
| `wake.presence.test.mjs` | Board presence | 3 | `crates/argus-wake/tests/presence.rs` | done |
| `wake.registry.test.mjs` | Wake registry and faults | 3 | `crates/argus-wake/tests/registry.rs` | done |
| `worktree.verbs.test.mjs` | coord worktree verbs | 8 | `crates/argus-coord/tests/worktree_verbs.rs` | done |
| `worktrees.fallback.http.test.mjs` | Worktree removal | 5 | `crates/argus-server/tests/worktrees_fallback_http.rs` | done |
| `worktrees.fallback.test.mjs` | Worktree removal | 5 | `crates/argus-worktrees/tests/fallback.rs` | done |
| `worktrees.http.test.mjs` | Worktree removal | 5 | `crates/argus-server/tests/worktrees_http.rs` | done |
| `worktrees.leftover.test.mjs` | Leftover folders | 5 | `crates/argus-worktrees/tests/leftover.rs` | done |
| `worktrees.remove.test.mjs` | Worktree removal | 5 | `crates/argus-worktrees/tests/remove.rs` | done |
| `worktrees.rules.test.mjs` | Worktree scanner; Leftover folders | 5 | `crates/argus-worktrees/tests/rules.rs` | done |
| `worktrees.safety.test.mjs` | Worktree removal | 5 | `crates/argus-worktrees/tests/safety.rs` | done |
| `worktrees.scan.test.mjs` | Worktree scanner | 5 | `crates/argus-worktrees/tests/scan.rs` | done |
| `worktrees.service.test.mjs` | Worktree scanner | 5 | `crates/argus-worktrees/tests/service.rs` | done |

A row with two slices is ported in two steps: the cases that need the later slice (owner posts in
`wake.http`, the native sender in `notify`, the native sampler in `procs.sampler`, the Rust client
in `runs.cli`) wait for it. `pages.portfolio` covers the Dashboards tab in slice 1 and the
Processes, Activity and Notifications tabs in slice 4; `pages.board` covers the page shell and
the Activity feed in slice 2 and the other tabs in slice 5.

## Open gaps by test file

### Markdown gaps (md.test.mjs, for the Tessera lead)

The cases are in `apps/web/tests/markdown/` as `it.fails` cases named `gap: ...`: they pass while
Tessera lacks the feature and fail the day it ships it, and then become plain `it` cases.

- Links to `/files/...` proof addresses: Tessera Markdown links only http, https and mailto.
- Inline images: every image shows its alt text (the Node page showed `/files`, https and data images).
- `linkIds`: `T-5`, `I-3`, `A-2` and `H-1` stay text (the Node page linked them to page anchors).
- Control and private-use characters pass through (Node removed them; NUL becomes U+FFFD).
- The editor lacks list continuation on Enter, Tab and Shift+Tab indent, Backspace on an empty
  marker and the checkbox autoformat of a dash and empty brackets.
- Different on purpose and asserted as Tessera does it: raw html is dropped (Node showed it as
  text), entities are decoded to text, headings start one level lower, a soft break stays a
  newline, a table cell carries its alignment as a class, a bold with no selection inserts the
  two marks and no placeholder word, and a text is never cut at a size cap.

### Board page gaps (pages.board.test.mjs, for the web lead)

All 110 Node cases were matched by title to the 627 web cases; three were added
(`note-limits.test.tsx`). These wait on a feature the web app does not have yet:

- The Tests tab note box and the note dialog are a plain Textarea; the Node box was the Markdown
  editor with a toolbar and list keys (4 cases: toolbar, every box the same editor, toolbar
  formatting, keys in the note box). Its over-limit sentence is also worded differently.
- The drawer: the criterion as inline markdown, the Expand focus toggle (Tessera Drawer wide) and
  the thread drawer with its note and brief (3 cases).
- The terminal's Copy all, Jump to bottom and 300 line window belong to Tessera LogPanel and need
  layout that jsdom lacks.
- Not applicable to React: the persistent body mounted into a slot, the redraw-free patch of a
  run line, and the page without `md.js`.

## Node fixes after the matrix commit

Node is frozen except fixes, and every fix reaches Argus. Each line names the `ai-config` commit and
where it landed.

| Node commit | Fix | Argus | Status |
|---|---|---|---|
| `db17ac5` | A deaf thread with an open fault tells the Overlord again (at most every 10 min); a `no-listener` fault clears itself when the thread listens | `crates/argus-wake/src/registry` (`faults.rs`, `watch.rs`, `pending.rs`), cases in `crates/argus-wake/tests/registry.rs` | ported |
| `db17ac5` | A worktree discard note counts only with a delete word, no negation and the worktree named | `crates/argus-worktrees/src/approval.rs` (word lists as in Node `remove.mjs`) | ported |
| `db17ac5` | Listener hook timeout 3 days (the rendered hook timeout; the listener keeps its 6 h default); master launcher for the Overlord project | `docs/coord-port.md` section 3; `crates/argus-coord/src/verbs/hook/listen` | ported |
| `62e66c7` | Sub-agent hooks bind by exact agent id: no word matching, a known id reopens a record closed for silence, an Agent result never rewrites another agent's record | `crates/argus-coord/src/verbs/hook` (`binding.rs`, `subagent_start.rs`, `subagent_stop.rs`, `agent_result.rs`), cases in `hook_agents*.rs` | ported |
| `62e66c7` | The reaper judges by the agent task data and worktree activity (opus level agents get 60 min, `agents.reapOpusMs`); the reconciler reopens a record the reaper closed while the agent still worked | not ported: `crates/argus-agents/src/reaper.rs` and `reconciler/resume.rs`; Node cases: 5 in `agents.reaper.test.mjs`, 3 in `agents.reconciler.test.mjs` | todo |

## Feature areas

One row per area, from `DESIGN.md`, `README.md`, `pages/`, `src/` and `static/`. Names are at most
40 characters and acceptance lines at most 120, so a board item can be made from each row. Areas
with no Node test (palettes, assets, shadow mode, the two shims, the switch-over) are proven by
their own Argus tests and by the owner's check.

| Feature area | Acceptance | Slice |
|---|---|---|
| Config and data folders | Argus reads and writes config.json, paths and env overrides as Node does and keeps unknown keys. | 1 |
| SQLite store and migrations | Argus opens a Node dashboard.db at version 6, refuses a newer one and applies migrations in order. | 1 |
| Doc store and change feed | Apply, replace, changes since and pruning give the same docs and seq rules as the Node DocStore. | 1 |
| HTTP security and errors | Loopback, Host, Origin, content type and token rules, error shape and body limits match Node. | 1 |
| Board alias hosts | Alias hosts, the reserved names and argus-app redirect and answer as on the Node server. | 1 |
| Doc stream SSE | Snapshot, ready, changes, reset, resume and filters are byte-compatible with the Node stream. | 1 |
| Board ingestion and export | Every real board exports the same docs as Node, and a CLI write reaches the stream within 1 s. | 1 |
| Portfolio cards | Projects and cards match Node for the real boards, master launcher and run counts included. | 1 |
| Server CLI and lifecycle | Serve, start, stop, status, open, resync and token act as the dashboard bin, with no console window. | 1 |
| Display names | Overlord and Argus names show where Node shows them; ids and scopes stay the same. | 1 |
| Web shell and data layer | The data layer passes the shim cases: snapshots, changes, resume, posts and reload on a newer build. | 1 |
| Overview Dashboards tab | The Dashboards tab lists every project card with its master Play, palette and board link. | 1 |
| Web assets serving | Release serves the embedded build with the page headers; debug and Vite dev serve the same app. | 1 |
| Shadow mode and parity diff | Shadow Argus runs on real data with no outward effect, and the parity diff finds no difference. | 1 |
| Board page shell | The board page renders header, switcher, tab bar with badges and the presence claim. | 2 |
| Board palettes | Each board wears its own Tessera palette, the overview stays basic, and a board switch follows. | 2 |
| Board Activity feed | The Activity tab shows All, Needs you and Log from the live docs with the Node page content. | 2 |
| Markdown rendering | Owner and thread text renders as markdown and passes the md.js URL allow-list and XSS cases. | 2 |
| Proof images and lightbox | Proof images load from the file route and open in the viewer with fit, zoom and pan. | 2 |
| Wake policy | Every row of the wake policy table yields the same wakes, reasons and texts as Node. | 3 |
| Wake engine and listeners | Windows, coalescing, queued replay, v2 listeners and the wakes mirror match Node. | 3 |
| Wake stream command | The wake-stream command prints the same lines, reconnects and exits 0 on replaced. | 3 |
| Wake registry and faults | Registry states, the three failures, one fault per thread and the Overlord-only clear match Node. | 3 |
| Board presence | Open board tabs are tracked per thread and board-closed faults open and clear as on Node. | 3 |
| Agent wakes | The agent-stale and agent-checkin wakes fire on the same clock and records as on Node. | 3 |
| Kanban stale reminders | Kanban reminders and the Overlord digest fire with the same thresholds, skips and texts. | 3 |
| Registered strip | Both pages show the Registered strip with the same chips, counts and tooltips as Node. | 3 |
| Activity log | Activity lines are written, capped and filtered as on Node and shown in the Activity tab. | 4 |
| Process rules and sampling | Rules load, reload and survive a bad edit, and samples list and flag the processes Node lists. | 4 |
| Process alerts and kills | Alerts coalesce into one wake, and kill and kill-rule obey the same guards and callers. | 4 |
| Overview Processes tab | The Processes tab lists, sorts, filters and kills as the Node page does, from the page only. | 4 |
| Notifications to the owner | Pings, levels, limits, automatic pings and the record of 50 behave as Node with a fake sender. | 4 |
| Overview Notifications tab | The tab lists, filters and badges pings, sends a test ping and edits toast and sound settings. | 4 |
| Board file locking | Argus takes, waits for and releases board.lock and saves with rename retry as the coord store. | 5 |
| Owner posts | Every reply and op kind applies with the Node status and outcome and is idempotent by id. | 5 |
| Owner text limits | Page text up to 50000 characters is stored and delivered whole; CLI and MCP keep short limits. | 5 |
| Portfolio send down | A send from the overview reaches the lead's board and inbox whole, with the same wake line. | 5 |
| Test results and proofs | Results and pasted proofs land on the board and show at once; fail needs a note. | 5 |
| Kanban tab | The Kanban shows, filters and moves items, hides done ones and marks quiet ones. | 5 |
| Tests tab | The Tests tab groups by bundle, records results, attaches proofs and hides closed bundles. | 5 |
| Runs lifecycle | Defined runs start, queue, stop, time out, recover and stay hidden as the Node runs do. | 5 |
| Run logs and terminal | Logs chunk, cap, redact and stream; the terminal shows colours, stdin and the last line. | 5 |
| Launchers and bundles | Launcher and bundle docs, Play defs, master, join, usage window and stale wakes match Node. | 5 |
| Builds tab | The Builds tab lists launchers with Play, Stop, terminal, Retired list and warnings. | 5 |
| Worktree scanner | Rules, scans, statuses and stale wakes give the Node rows and texts on throwaway repos. | 5 |
| Worktree removal | Every removal guard, backup, branch decision and folder fallback answers as Node. | 5 |
| Leftover folders | Leftovers are measured, listed and deleted behind the same guards and jobs as Node. | 5 |
| Worktrees tab | The Worktrees tab shows rows, confirms, requests, leftovers and jobs as the Node page. | 5 |
| Agents live view | Transcript tailing fills the live agent fields on the docs and never in board.json. | 5 |
| Agent reaper | Quiet hook records close after the reap delay with the same note and activity line. | 5 |
| Agent reconciler | Linking, duplicates, idle and orphan classes, reopen and the report route match Node. | 5 |
| Agents tab | The Agents tab shows launches, live runs, agents with model and why, and finished runs. | 5 |
| Markdown editor | The owner box edits markdown with toolbar, list keys and Ctrl+Enter through the Tessera editor. | 5 |
| ANSI terminal rendering | Terminal output renders colours and control sequences as the ansi.js cases expect. | 5 |
| Native toasts | Toasts show in-process through WinRT with the same XML, tag and fallback, with no PowerShell. | 6 |
| Tray icon and menu | The tray shows the icon and Windows 11 menu, starts and stops the server and honours quit.flag. | 6 |
| Native process sampling | The native sampler lists the same processes, memory, CPU and hung state as the PowerShell one. | 6 |
| MCP tool host | All 23 tools and the resources answer as Node on the fixture, with allow, deny and readOnly. | 7 |
| MCP HTTP transport | The MCP route answers initialize, lists, calls and notifications with the Node codes and headers. | 7 |
| MCP stdio bridge | The bridge answers initialize locally, forwards calls, starts the server and keeps stdout pure. | 7 |
| MCP install and render | install-mcp, autostart and the rendered .mcp.json keep working with the dashboard command. | 7 |
| Dashboard command shim | The dashboard command and the installDir bin both run argus with the same arguments. | 7 |
| Switch-over to port 47615 | Argus serves 47615 on the real data home, every client works unchanged, and rollback works. | 7 |
| coord CLI shim | The coord command on PATH runs argus coord, and every hook keeps working unchanged. | 8 |
| coord agent verbs | Agent start, set, list and show enforce model, why and categories as the Node CLI. | 8 |
| coord session hooks | Every coord hook (guard, listen, autostart, agents, transcript) behaves as the Node hooks. | 8 |
| coord build and launcher verbs | Build and launcher verbs write the same records, defs and refusals as the Node CLI. | 8 |
| coord worktree verbs | Worktree rule, keep, done, remove, prune and leftovers verbs behave as the Node CLI. | 8 |
| coord lead, inbox and ping | Lead, inbox send and ping deliver, wake and refuse as the Node verbs. | 8 |
| coord entry context rules | Ask, decide, notify and message require context and options as the Node verbs. | 8 |
| coord board shortcut | Board shortcut writes launch.json entries on alias hosts as the Node verb. | 8 |
| coord run client | Run verbs start, wait, stop, list and stream logs through the server and start it when down. | 8 |

## Switch-over checklist

- [ ] Every matrix row is `done` or `n/a`, and every feature area of slices 1 to 7 is accepted.
- [ ] The Node commit the matrix was last checked against is the current `ai-config` HEAD.
- [ ] `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo test --workspace` pass with `ARGUS_COORD_CLI` set.
- [ ] `pnpm lint`, `pnpm lint:md`, `pnpm structure` and `pnpm test` pass.
- [ ] Shadow Argus ran beside Node on the real data for a full day, and `argus parity` reports no
  difference at its end.
- [ ] The MCP `tools/list` of Argus equals the Node one, name for name and schema for schema.
- [ ] `render.mcp` and `render.publish` pass in `ai-config` with the `dashboard` shim in place.
- [ ] A copy of `dashboard.db` is taken (`dashboard.db.node-<stamp>`).
- [ ] The Node server and its tray are stopped through `dashboard stop` and `dashboard tray stop`,
  never by process name.
- [ ] The `dashboard` shim and `<installDir>/bin/dashboard.mjs` start `argus.exe`.
- [ ] Argus runs one hard resync on start and serves port 47615.
- [ ] Every live session's `wake-stream` listener has reconnected, and wakes queued before the cut
  replay to it.
- [ ] A post from the board page, a `coord item add`, a `coord run start --wait`, a `coord ping`
  and an MCP `board_summary` all work against Argus.
- [ ] The alias hosts and the board shortcuts open their boards.
- [ ] The tray icon is back and a test ping shows a toast.
- [ ] The rollback was rehearsed in shadow: Node started on a database Argus had written.
- [ ] The owner has the launch command and tests Argus himself; the Node dashboard folder stays in
  place until he confirms.
