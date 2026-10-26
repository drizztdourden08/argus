<!-- @layer docs @kind doc -->
# coord port

Slice 8 moves the `coord` CLI from Node (`X:/ai-config/shared/tools/coord`) into Argus as
`argus coord <verb>`. Every hook, every rendered project and every typed command keeps working
with no change to the hook config. The hooks keep calling
`node .claude/tools/coord/coord.mjs hook <event>`, and that file forwards to `argus.exe`. The exit line is the slice 8 row of
`docs/parity.md`: the coord suites pass against `argus coord`, and the `coord` shim calls it.

The specification is the Node CLI at `ai-config` commit `4dbe260` (the coord folder last changed
in `ff939e2`). It is owned by the Overlord: every change on the `ai-config` side named here is a
request through `coord lead portfolio`, never a local patch.

## Ground rules

- Same files, same bytes. `argus coord` writes `board.json` byte for byte as Node does (key
  order, two-space JSON, number forms, the trailing newline), under the same lock.
- Same text. Every stdout and stderr line, every refusal and every exit code is the Node one.
  The differential harness of the test plan is the judge.
- Port the case, not the code, as for the server. The Node verb files are read for what they
  promise.
- The board semantics exist once in Rust: `argus-board` already carries the store, the lock, the
  ops, the limits, pause, the portfolio delivery and the entry import. `argus-coord` calls them
  and adds only the CLI layer.

## 1. Inventory

### Entry and routing

`coord.mjs` parses argv, picks a module and maps errors to exit codes. Its rules, all of which
the Rust router keeps:

| Rule | Behaviour |
|---|---|
| Arguments | `cli-args.mjs`: `--k v` and `--k=v`; a `--k` followed by a word that does not start with `--` takes it as its value; a repeated option becomes a list; a bare `--k` is `true`. Ported by hand, never with `clap` |
| `hook <event>` | Runs before any project lookup, inside a catch that swallows everything: exit 0 and no output on any error |
| `portfolio`, `dashboard` | Need no project |
| `wake-fault`, `proc`, `activity` | Need no project; find their caller when they need one |
| `lead` | Finds its own sender, with its own refusal text |
| `help`, `--help`, no verb | Prints `USAGE` on stdout, exit 0 |
| Unknown verb | stderr `Unknown verb "<v>".`, a blank line and `USAGE`, exit 1 |
| Aliases | `ask`, `decide`, `notify`, `message`, `answer`, `entry` go to the entry module with the typed verb; `todo` is `action`; `restore`, `trash` go to the purge module with the typed verb; `dispatch` is `thread add` |
| Project | `--project`, `COORD_PROJECT`, the board holding a live thread with this session id (every board under `COORD_HOME` is read, in directory order), the nearest `.ai-config.json` `projectKey`, a `.coord-project` marker, else `cannot tell which project this is: no .ai-config.json above the cwd. Pass --project <key>.` |
| Caller | `--as`, the live thread holding `CLAUDE_CODE_SESSION_ID`, else the live thread whose `root` holds the cwd and that no other session claimed (longest root wins, a tie refuses). A thread never acts as another thread; the coordinator may, and is recorded as the actor. Every verb refreshes the real caller's `lastSeen` |
| Output order | A verb prints inside its board mutation, before the save. A failed save then prints `coord: ...` after the verb's own lines. Rust keeps that order |

### Exit codes

| Code | When | Stream |
|---|---|---|
| 0 | Success, help, a `retired: ...` notice, every hook except a firing listener | stdout |
| 1 | Any refusal: stderr `coord: <message>`. Also a server refusal in `activity`, `ping`, `proc`, `wake-fault`, `worktree` (printed, then exit 1), and `proc kill-rule` with a failed kill | stderr |
| 2 | `hook listen` heard a wake: stderr `WAKE <thread>: <lines>[ (<n> wakes)]. Run: coord inbox` | stderr |
| 3 | `run` verbs when the dashboard will not come up: `coord: the dashboard is not running (dashboard start)[: <detail>]` | stderr |
| n | `run start --wait`: the run's own non-zero exit code, or 1 for a failure without one. `dashboard <x>`: the dashboard bin's code | both |

### Files

| File | Who touches it |
|---|---|
| `<COORD_HOME>/<key>/board.json` | Every board verb and the six board hooks. Read, normalized (schema 1 to 2, embedded agents to agents docs), mutated, saved through `board.json.<pid>.tmp` and a rename retried 8 times 25 ms apart |
| `<COORD_HOME>/<key>/board.lock` | Every mutation: create-new, stale after 15 s, the CLI waits 5 s polling every 50 ms, then `board is locked by another coord call (<path>); retry.` |
| `<COORD_HOME>/<key>/board.v1.<stamp>.json`, `board.pre-agents.<stamp>.json` | The first save of a board that needed converting (written with create-new) |
| `board.inbox[<thread>]` | The inbox lives inside the board: `inbox`, `lock`, `log`, `entry`, `thread pause`, `lead`, `portfolio send`, the hooks (which mark `deliveredAt`) |
| `board.agentDispatches`, `agentStops`, `agentLaunches` | The four sub-agent hooks and `agent start` (kept 20, TTL 10 min, launches 60 min) |
| `<COORD_HOME>/_portfolio/board.json` | `portfolio`, `lead` (known projects), `lead` and `portfolio send` deliveries |
| Session registry | No file of its own: `threads[].sessionId` across all boards, plus `<DASHBOARD_HOME>/listen-<project>-<thread>.json` (the listener's pid and start) |
| `<DASHBOARD_HOME>/config.json`, `server.pid` | Read for port, token, `requireToken`, `installDir` |
| `<DASHBOARD_HOME>/quit.flag`, `start.lock` | Read by the autostart of the session-start and prompt hooks; `start.lock` written |
| `<checkout>/.claude/launch.json` | `board shortcut` writes it; the session-start hook reads it |
| `<checkout>/.claude/settings.json` | The session-start hook reads it (does it carry `hook listen`) |
| `<checkout>/.ai-config.json`, `.coord-project`, `.git` | Project and root lookup |
| Sub-agent transcripts | `subagent-stop` reads the first 512 KB of `agent_transcript_path` and its `.meta.json` |
| `<dir>/entries/*.json` | `sync import <dir>` |
| `<backup.json>` | `restore --from` |

Environment read by the CLI: `COORD_HOME`, `DASHBOARD_HOME`, `DASHBOARD_PORT`, `COORD_PROJECT`,
`CLAUDE_CODE_SESSION_ID`, `CLAUDE_CODE_HOST_SESSION_ID`, `COORD_LISTEN_MS`,
`COORD_DASHBOARD_WAIT_MS`, `COORD_DASHBOARD_INSTALL`, `COORD_DASHBOARD_NO_AUTOSTART`,
`COORD_DASHBOARD_AUTOSTART`.

### Text limits

`limits.mjs`, already ported as `argus-board::limits`. A text is trimmed, then refused empty
(`<kind> text is required.`) or long (`<kind> is <n> chars, the limit is <max>. Shorten it.`).
Lengths are JavaScript lengths, in UTF-16 code units, and `clipText` cuts at a code unit and adds
a horizontal ellipsis character.

| Kind | Max | Kind | Max | Kind | Max |
|---|---|---|---|---|---|
| name | 40 | subject | 40 | note | 100 |
| title | 80 | description | 400 | ac | 120 |
| question | 240 | option | 60 | answer | 600 |
| followup | 240 | lead | 240 | ping | 240 |
| decision | 200 | notice | 140 | message | 240 |
| action | 200 | log | 120 | brief | 1200 |
| task | 120 | covers | 80 | context | 600 (min 20 where required) |
| changed | 240 | label | 40 | setup | 300 |
| command | 300 | step | 240 | expect | 200 |
| testnote | 300 | caption | 120 | result | 300 |
| reason | 120 | model | 40 | modelwhy | 200 |
| branch | 80 | worktree | 120 | agentnote | 100 |
| agentresult | 200 | build | 40 | buildnote | 600 |
| buildcmd | 400 | buildcwd | 260 | launchercmd | 400 |
| launchercwd | 260 | launchernote | 300 | commit | 40 |

Lists (`--ac`, `--option`, steps, expects) hold at most 20 entries. A ping title is at most 60.

### Verbs

Each row names the Node module, so the full set of strings is one file away. The work package
(WP) column is the split of section 2.

| Verb | Sub-verbs | Limits used | Output and refusals | Touches | WP |
|---|---|---|---|---|---|
| `hook` | session-start, prompt, tool, stop, guard, agent-dispatch, agent-result, subagent-start, subagent-stop, listen | clips at 60 to 1000 | See section 3 | board, listen stamp, autostart files, transcripts | 2 |
| `thread` | register, note, status, flag, done, list, show, set, add, rm, pause, resume, archive, unarchive | name, subject, note, brief, reason, title | `registered thread <n> (root <r>)`, tips, `<id>: done`, lock hand-over lines, tables; `thread verb must be one of ...` | board, other boards (session lookup) | 3 |
| `dispatch` | (same as `thread add`) | name, subject, brief | `dispatched <n> (<s>). Chip title: ...` plus the prompt | board | 3 |
| `board` | init, name, prefixes, show, open, path, page, url, shortcut | none | summaries, `retired: ...`, launch.json lines | board, `.claude/launch.json` | 3 |
| `purge`, `restore`, `trash` | thread, `--keep`, `--status`, `--idle`, `--trash`; restore `--from`; trash list | reason | dry run with the exact confirm command; counts | board, backup JSON | 3 |
| `check` | `--stale`, `--json` | none | `# check <key> ...` report or JSON | board (read) | 3 |
| `tick` | (none), woke | none | JSON `{ wake, attention }`; `woke: ...` | board | 3 |
| `sync` | import; retired export, pushed, versions, asset | none | JSON import result; `retired: ...` | board, entries dir | 3 |
| `item` | add, set, status, move, ac (add, set, done, rm, move), ac-done, ac-rm, archive, unarchive, rm, restore, show, list | title, description, subject, ac, reason | `<I>: <title> (<owner>, <n> AC)`, `(rev n)` lines, tables, JSON | board | 4 |
| `test` | add, set, step, setup, expect, proof, result, move, soft verbs, show, list | title, label, step, setup, expect, command, testnote, caption, result, changed, covers, context, build | result lines, tables, JSON; fail needs a note | board, proof files | 4 |
| `ask`, `decide`, `notify`, `message`, `answer`, `entry` | entry: list, show, pick, done, set, move, soft verbs | question, option, decision, notice, message, answer, context | the context teaching text when missing; 2 to 5 options for a decision | board, inbox | 4 |
| `action`, `todo` | add, set, done, move, soft verbs, show, list | action, reason | `no manual actions.`, tables | board | 4 |
| `lock` | claim, release, drop, free, rm, list | description | `<r>: you left the queue.`, `no locks.` | board, inbox | 4 |
| `log` | (write), list, rm | log, subject | `<s> #<n>: removed.`, an `audit` line for another's line | board, inbox | 4 |
| `inbox` | (read), `--ack`, list, rm, send | followup | `<t>: inbox empty.`, message lines, `queued for <t> (session "...")`; send wakes over HTTP, best effort | board, `POST /api/p/:key/wakes` | 4 |
| `agent` | start, note, set, done, fail, stop, archive, unarchive, rm, restore, list, show, reconcile | name, task, model, modelwhy, agentnote, agentresult, branch, worktree, reason | `<A>: agent <n> started on <t> (...)`, the model standard and `--why` refusals, check-in rule | board, `GET /api/agents/reconcile` | 5 |
| `build` | add, set, rm, close, reopen, ready, draft, hand, unhand, list | build, buildnote, buildcmd, buildcwd, title | `handed to the owner: ...`, tables, 29 refusals | board (builds, launchers) | 5 |
| `launcher` | set, keep, retire, list, show | launchercmd, launchercwd, launchernote, branch, commit | show lines, tables, refusals for master and windows | board (launchers) | 5 |
| `run` | define, undefine, defs, start, stop, status, list, logs | command, reason | run summaries, `--wait` and `--follow` streams, exit 3 when down | board (runDefs), runs API | 6 |
| `ping` | (text) | ping, title 60 | the notify sentence | `POST /api/notify` | 6 |
| `activity` | `--limit`, `--kind`, `--level`, `--json` | none | `no activity.` or lines | `GET /api/activity` | 6 |
| `proc` | list, rules, rule set, rule rm, kill, kill-rule, help | reason | tables, kill summaries | procs API | 6 |
| `wake-fault` | list, clear | note | `cleared the wake fault of ...`; Overlord only | wake-faults API, portfolio board | 6 |
| `worktree` | rule add, rule rm, rules, list, leftovers, keep, done, remove, prune | note, reason | tables, server texts | board (rules, keeps, answers), worktrees API | 6 |
| `lead` | `<key> "<text>"`, `--kind note or decision` | lead | `queued for <key> lead: <text>` | target board, `_portfolio`, wakes API | 6 |
| `portfolio` | init, add, remove, send, tick, show; retired page, url, pushed | name, kind texts | cards table, `retired: ...` | `_portfolio`, every board on send | 6 |
| `dashboard` | every dashboard verb | none | passthrough | runs the dashboard bin | 6 |

## 2. Work packages

Every package lands in `crates/argus-coord` (it exists as a plan row only today), each owning its
own folder so parallel agents never edit the same file. WP1 lands first and alone: it creates
every shared file and a stub for every verb group. WP2 to WP7 then run in parallel.

```text
crates/argus-coord/
  Cargo.toml                 WP1 (every dependency declared up front)
  src/lib.rs                 WP1: run(Invocation) -> i32, the in-process entry
  src/cli/                   WP1: args, route (the VERBS table), usage, out (say, table, ago,
                             clip, json), exit (error to code), env (paths, clock, session)
  src/who/                   WP1: project lookup, caller, me, session scan
  src/client/                WP1: dashboard client, health, autostart, SSE reader
  src/shortcut.rs            WP1: alias urls, launch.json read and merge, needs_shortcut
  src/verbs/hook/            WP2
  src/verbs/thread/          WP3: thread, dispatch, board, purge, check, tick, sync
  src/verbs/work/            WP4: item, test, entry, action, lock, log, inbox
  src/verbs/agent/           WP5: agent, build, launcher
  src/verbs/remote/          WP6: run, ping, activity, proc, wakefault, worktree, lead,
                             portfolio, dashboard
crates/argus/src/main.rs     WP1: the coord fast path
crates/argus/tests/coord_*   WP7: end-to-end and differential runs of the binary
crates/argus-testkit/fixtures/coord/   WP7: corpus and twin-home seeds
```

Each group's `mod.rs` starts as a stub answering exit 120, "not ported" (section 4), so the router
compiles from day one, the same way `services/compose.rs` works in the server.

| WP | Content | Reuses | Tests (parity rows) |
|---|---|---|---|
| 1 Foundation | Crate, `Invocation` (argv, env, cwd, stdin, stdout, stderr, clock), the hand port of `parseArgs`, the router with every verb and alias, `USAGE` text, output helpers with JavaScript widths, project and caller lookup, the dashboard client (port from `server.pid` then `config.json`, bearer only with `requireToken`, autostart with an 8 s wait), the SSE reader, `shortcut.rs`, the error to exit code map, the `argus coord` fast path in `main.rs`. Also opens the `argus-board` surface the others need (the inbox `post`, pause, agent helpers) in one change | `argus-board` store, lock, limits, names; `argus-core` paths, `js` | `board.shortcut` helper cases, arg parsing cases, project and caller cases from `hook.guard` and `lead.verbs` |
| 2 Hooks | The ten hook events, visit and adopt, the context texts, guard, the sub-agent bindings, the listener, the autostart | `argus-agents` `hook_logic`, `text` (bestFit), `model_rules` | `hook.agents`, `hook.agents.binding`, `hook.agents.claudeid`, `hook.autostart`, `hook.guard`, `hook.listen`, `hook.transcript` (80 cases) |
| 3 Threads and board | thread, dispatch, board, purge, restore, trash, check, tick, sync import | `argus-board` ops, pause, backup, sync import | thread and board cases of `owner.text`, `sync.watch` CLI cases, `board.shortcut` verb cases |
| 4 Work docs and messages | item, test, entry and its aliases, action, lock, log, inbox | `argus-board` ops (create, edit, ac, move, soft, result), limits, results, proofs | `entry.context` (10), `inbox.wake` (2), the CLI cases of `notify.http` for `ask` |
| 5 Agents and bundles | agent (with the hook binding of `agent start`), build, launcher | `argus-agents` model rules and records; `argus-launchers` launcher record, write, every, retire | `agent.model` (12), `build.verbs` (23), `launcher.verbs` (16) |
| 6 Server clients | run, ping, activity, proc, wake-fault, worktree, lead, portfolio, dashboard (passthrough becomes an in-process call of the `argus` subcommand) | `argus-board` portfolio deliver and cards; `argus-worktrees` rule parsing; `argus-notify` sentence | `runs.cli` (6), `lead.verbs` (6), `worktree.verbs` (14), `wake.faults` CLI cases, `notify.http` ping cases, `portfolio.longsend` CLI cases, `procs.http` CLI cases |
| 7 Shim, harness, bench | The forward block for `coord.mjs` (drafted here, landed by the Overlord), the differential harness, the latency bench, the twin-home seeds and corpus | `argus-testkit` | The harness itself; the Node slice 8 suites run with `COORD_ENGINE=argus` |

Rules for the parallel run:

- A package edits only its own folder. A missing `argus-board` function is reported to the lead,
  who adds it in one serialized change; no package copies board logic into its folder.
- `Cargo.toml`, `lib.rs`, `cli/`, `who/`, `client/` and `main.rs` belong to WP1 and are frozen
  once it lands. A package that needs a new helper there asks the lead.
- Every package ports its Node cases as in-process tests in `crates/argus-coord/tests/`, named as
  `docs/parity.md` says, calling `argus_coord::run` with a temp `COORD_HOME`, a fixed session id
  and captured streams. Only WP7 spawns the binary.
- Dependencies: `argus-coord` depends on `argus-core`, `argus-board`, and for their pure modules
  `argus-agents`, `argus-launchers`, `argus-worktrees` and `argus-notify`, so each rule exists
  once. The architecture row for the crate changes from "core, board" to that list; there is no
  cycle, since no service crate depends on `argus-coord`. HTTP goes through `ureq` with no TLS
  (blocking, no runtime to start), which replaces `reqwest` in the coord row of the libraries
  table.

## 3. The hook path

### Contracts

Every hook reads one JSON object on stdin (an unreadable one counts as `{}`), returns at once
when its `cwd` does not exist, and exits 0 with nothing on stdout on any failure. Output is
written with no trailing newline.

| Event | Hook | Reads | Stdout |
|---|---|---|---|
| SessionStart | `session-start` | `session_id`, `cwd`; `CLAUDE_CODE_HOST_SESSION_ID` | JSON `hookSpecificOutput` `SessionStart` with `additionalContext`: the join text for an adopted session, the placeholder reminder, or the thread line, the board pane line, the shortcut line and the wake line; then the new messages |
| UserPromptSubmit | `prompt` | same | Plain text (not JSON) when adopted, still a placeholder, or messages wait; else nothing |
| PostToolUse | `tool` | same | JSON `PostToolUse` `additionalContext` only when adopted or messages wait |
| Stop | `stop` | same | JSON `{ "decision": "block", "reason": <messages> }` when messages wait; never adopts |
| PreToolUse `tabs_close` | `guard` | same | JSON `PreToolUse` deny with the board url, for a registered thread only |
| PreToolUse `Agent or Task` | `agent-dispatch` | `tool_name`, `tool_input` (model, description, prompt, subagent_type, run_in_background), `tool_use_id` | nothing |
| PostToolUse `Agent or Task` | `agent-result` | `tool_response` (`agentId` field or the `agentId: <id>` line) | nothing |
| SubagentStart | `subagent-start` | `agent_id`, `agent_type`, `transcript_path` | nothing |
| SubagentStop | `subagent-stop` | `agent_id`, `agent_transcript_path`, `last_assistant_message` | nothing |
| SessionStart, Stop, PostToolUse (async, asyncRewake) | `listen` | same as `session-start` | nothing; stderr wake line and exit 2 |

The board side of `session-start`, `prompt`, `tool` and `stop` is one locked read, modify and
write: heartbeat (`lastSeen` when older than 60 s or the activity changed), activity (`working`,
or `idle` on Stop), `sessionId` follow, `deliveredAt` on every handed message, and the adoption
of an unknown session as a placeholder `s-<6 chars>`. A session whose threads here all ended is
left alone. `session-start` and `prompt` also run the autostart: when no live pid holds
`server.pid`, no `quit.flag` is newer than the boot, no `start.lock` is younger than 60 s and no
home override is set, write `start.lock` and start the tray detached and hidden. After the
switch-over the Rust hook starts its own executable with `tray start` (`current_exe`, with
`DETACHED_PROCESS` and `CREATE_NO_WINDOW`).

### Latency budget

Measured on the owner's machine (Node 24, a copy of the real home: 11 boards, 2.8 MB of
`board.json`): a bare `node -e ""` takes 47 ms, and `hook tool`, `hook prompt` and
`hook session-start` take 76 to 80 ms each. Every tool call fires `hook tool` and `hook listen`,
two Node starts.

| Path | Target p50 | Target p95 |
|---|---|---|
| `argus coord hook tool`, `prompt`, `stop`, direct | 15 ms | 40 ms |
| `argus coord hook session-start`, direct | 25 ms | 50 ms |
| The same hooks through the `coord.mjs` shim | Node today plus 10 ms | Node today plus 20 ms |

How the binary meets it:

- `main` checks `argv[1] == "coord"` before anything else: no `tokio` runtime, no `clap`, no
  logger, no config read unless the verb needs it. The HTTP verbs use blocking `ureq`.
- The session scan never parses every board: each `board.json` is searched for the session id
  as bytes first, and only a board that contains it is parsed, into a typed peek of
  `threads.*.sessionId` and `status`. The directory order is the OS order, as in Node.
- Only the target board is parsed in full (`serde_json::Value` with `preserve_order`) and saved
  once. Node saves on every visit of a known thread, which moves `updatedAt`; Rust does the same
  for parity. Skipping a save that changes nothing is a later optimization for the lead and the
  Overlord to agree on, since the dashboard reacts to every save.
- A bench in WP7 measures p50 and p95 of each hook over 200 runs on a copy of the real home and
  fails above the targets. A miss after these steps leads to the fallback of a slim second binary
  built from `argus-coord` alone (risk 1).

### Never break a session

The Rust hook path runs inside `catch_unwind`, maps every error to exit 0 with no output, and
holds the same 5 s lock wait as Node, under the hooks' 10 s timeout. A panic never reaches the
session host as exit 101.

### The asyncRewake listener

`hook listen` runs for up to `COORD_LISTEN_MS` (6 h). Its contract, kept as is:

- Returns 0 for no board, no session, a placeholder or an ended thread.
- On PostToolUse it returns 0 when `listen-<project>-<thread>.json` names another live pid
  started in the last 120 s; otherwise it writes that file with its own pid and start.
- A lead listens to `*`, a thread to its own id, labelled with its session title (80 chars).
- It opens `GET /api/p/<key>/wakes/stream?thread=<t>&v=2&id=<label>`. Event `replaced` exits 0.
  Event `wake` prints `WAKE <thread>: <lines joined by "; ">[ (<n> wakes)]. Run: coord inbox` on
  stderr (or `WAKE <thread>: see coord inbox` for bad data) and exits 2.
- A refused or broken stream runs the autostart and retries 5 s later, until the deadline.

Node commit `db17ac5` raised the hook timeout of the three `hook listen` entries in
`shared/hooks.json` (PostToolUse, SessionStart, Stop) from 21600 s (6 h) to 259200 s (3 days). WP2
reads `COORD_LISTEN_MS` against that budget: the listener must keep running as long as the hook
may, and the rendered config of Argus carries the same 259200. The same commit added the `master`
launcher of the Overlord project, which is project config and needs no port. The wake side of that
commit (the re-alert for a deaf thread, the self-clearing `no-listener` fault) is already in
`argus-wake`.

Under the shim, a Node parent waits on the Rust child for those 6 h. The shim passes its pid in
`ARGUS_COORD_PARENT`, and the Rust listener checks it every 5 s and exits 0 when the parent is
gone, so a killed hook leaves no orphan holding a wake stream. The server keeps one listener per
thread, so a leftover is replaced at the next start in any case.

## 4. The shim

### Today

```text
hooks.json      node .claude/tools/coord/coord.mjs hook <event>
Bash calls      .claude/tools/coord/coord <verb>    (bash: exec node coord.mjs)
PATH            ~/.rotp-cli/bin/coord, coord.cmd    (walk up to a checkout, node coord.mjs)
```

All three end in the rendered `coord.mjs`. That one file is the place to forward from, so the hook
config, the bash shim and the PATH shims stay as they are.

### The forward block

A new `forward.mjs` beside `coord.mjs` in `shared/tools/coord`, called on the first line of
`main()`, before any verb module loads. Its decision, in order:

1. `COORD_ENGINE=node` runs Node. `COORD_ENGINE=argus` must forward (tests), to `ARGUS_EXE` when
   set, else to the `exe` of `config.json`: a missing exe is an error, never a silent fallback.
2. Otherwise read `<DASHBOARD_HOME>/config.json`: `coordEngine` (`node` until the switch-over)
   and `coordVerbs` (the top-level verbs Argus serves, or `["*"]`). `exe` is the absolute path of
   `argus.exe` that Argus already rewrites on every start.
3. Forward when the engine is `argus`, the verb is listed and the exe exists:
   `spawnSync(exe, ["coord", ...argv], { stdio: "inherit", windowsHide: true })`, then exit with
   its status. Stdin, stdout and stderr are inherited, so hook input and output pass untouched.
   `hook listen` uses `spawn` with the parent pid in `ARGUS_COORD_PARENT` and forwards the
   child's exit code.
4. Exit 120 from `argus coord` means "not ported here": Argus returns it before reading stdin or
   touching any file, and the shim then runs the Node verb. Any other code is final: a verb that
   ran is never run twice.
5. A spawn error (no exe, access denied) runs Node, and appends one line to
   `<DASHBOARD_HOME>/logs/coord-shim.log`; for a hook it stays silent on stdout.

The cost is one small file read on every call; the Node start remains. Rollback is a config
edit: `coordEngine: "node"`.

### Phases

| Phase | State | Who |
|---|---|---|
| A | `forward.mjs` rendered into every project, `coordEngine` unset (Node) | Overlord lands and renders |
| B | `coordEngine: "argus"` with `coordVerbs` growing one group at a time: server clients, then work docs, agents and bundles, threads and board, hooks last | Argus lead, each step after its harness run |
| C | `coordVerbs: ["*"]`; the Node verb modules frozen, kept as the fallback until the owner confirms | Argus lead, then the Overlord |
| D | Optional: `hooks.json` and the PATH shims call `argus.exe coord` directly, which drops the Node start (the latency win); the Node verb modules leave the render, `coord.mjs` stays a forward stub | Overlord, with the bench numbers |

During phase B a checkout that has not been rendered since phase A runs Node whatever the config
says. The engine switch waits for a render that reached every project.

### What needs the Overlord's agreement

- Landing `forward.mjs` and its call in `coord.mjs`, rendered everywhere, with the engine off.
- The contract: the config keys `coordEngine` and `coordVerbs`, the env `COORD_ENGINE`, exit 120,
  `ARGUS_COORD_PARENT`, the shim log.
- A coord freeze from the start of WP2 to WP6, or a rule that each coord change ships with its
  Argus port and a corpus line, so the spec stops moving under the port.
- Running the Node slice 8 suites in `ai-config` with `COORD_ENGINE=argus` as part of the gate.
- Who owns coord code after phase C: the method and the skill stay the Overlord's, the Rust code
  lives in this repo, and a verb change becomes a request to the Argus lead.
- Phase D: the `hooks.json` commands, the `install-cli.mjs` shims (whose `coord.cmd` starts Node
  twice today), and removing the Node verb modules from the render.
- `dashboard-client.mjs` reading `exe` from `config.json` (already an open question in
  `docs/architecture.md`).

## 5. Test plan

### What exists

coord has no test folder of its own: its suites are the Node dashboard tests that spawn
`node shared/tools/coord/coord.mjs` against a temp `COORD_HOME` and `DASHBOARD_HOME`.

| Node file | Cases | Needs a server |
|---|---|---|
| `hook.agents`, `hook.agents.binding`, `hook.agents.claudeid` | 21, 11, 15 | no |
| `hook.guard`, `hook.autostart`, `hook.transcript` | 17, 5, 4 | no |
| `hook.listen` | 7 | yes |
| `agent.model`, `build.verbs`, `launcher.verbs` | 12, 23, 16 | no |
| `entry.context`, `board.shortcut` | 10, 12 | no |
| `worktree.verbs`, `lead.verbs`, `inbox.wake`, `runs.cli` | 14, 6, 2, 6 | yes |
| CLI cases of `notify.http`, `wake.faults`, `sync.watch`, `owner.text`, `portfolio.longsend`, `procs.http`, `tray` | part of each | yes |

That is 181 cases in the dedicated files plus the CLI cases of the shared ones.

### Four layers

1. Rust ports. Each package ports its rows as in-process tests in `crates/argus-coord/tests/`,
   with the `//!` header and snake-case names of `docs/parity.md`. Cases that need a server use
   `argus-testkit` and its ephemeral port, and point `DASHBOARD_HOME` at its `server.pid`.
2. The Node suites against Argus. Every Node case spawns `coord.mjs`, so once the forward block
   exists, `COORD_ENGINE=argus ARGUS_EXE=<target>/argus.exe node --test` on the slice 8 files runs
   the real Node specification against `argus coord`, unchanged. A file passes on both engines
   before its row turns `done`.
3. The differential harness (`crates/argus/tests/coord_diff.rs`, skipped with a reason when
   `ARGUS_COORD_CLI` is unset). Two twin homes are seeded from
   `crates/argus-testkit/fixtures/coord/` (the `alpha`, `beta` and `_portfolio` boards plus a
   hook scenario board). A corpus file holds one command per line, with directives for the
   session id, the cwd, stdin JSON and extra env. Each line runs with `COORD_ENGINE=node` on one
   home and `argus coord` on the other, in order, and the harness compares the exit code, stdout,
   stderr, and every file under both homes after each line (`board.json`, backups, inbox,
   `launch.json`, listen stamps). Comparison is on the bytes after normalizing ISO timestamps,
   file stamps, `m-<epoch ms>-` ids, relative times (`12s ago`), temp paths and pids, so key order
   and number forms are checked too. The corpus is generated from `USAGE` (every sub-verb, each
   limit at its max and one over, each refusal) and grown by hand; a verb with a server runs
   both sides against one test server each.
4. Real usage replay. A local script collects the `coord ...` command lines of the owner's past
   sessions into a corpus outside the repo, and runs layer 3 on two copies of the real
   `~/.coord`. It is never committed (it carries board text) and never touches the real home.

The bench of section 3 runs with layer 3 on every package landing.

### Gates

- A package is done when its Rust ports pass, its Node files pass with `COORD_ENGINE=argus`, and
  its corpus lines show no difference.
- A phase B step needs layers 2 and 3 green for its verbs and one day of layer 4 on fresh copies.
- Phase C needs every slice 8 row `done`, the bench green, and the full Rust and Node suites
  through the haiku test agent.

## 6. Order and risks

### Order

1. Ask the Overlord for the shim contract and the freeze (one `coord lead portfolio` message,
   non-blocking; the build starts on the recommended defaults).
2. WP1 alone.
3. WP2 to WP7 in parallel, one sub-agent each, each in its own worktree. One haiku test agent per
   finished package.
4. Phase A once the Overlord lands the forward block.
5. Phase B in the order server clients, work docs, agents and bundles, threads and board, hooks.
6. Phase C, then phase D if the Overlord agrees and the bench shows the gain.

### Risks

| Risk | Effect | Mitigation |
|---|---|---|
| 1. Process cost on Windows. Under the shim every call is a Node start plus an exe start; Defender scans a new, large `argus.exe` (embedded web build) on its first runs; the listener holds two processes for 6 h | Hooks slower than today, a slower prompt | The bench gate; the parent watch; phase D to drop Node; a slim `coord` binary from `argus-coord` alone if the big exe misses the target |
| 2. Byte parity. JSON number forms, key order, string escapes, UTF-16 lengths in limits, padding and clipping, the print-before-save order | The dashboard re-exports or a board diff on every write, a refusal text that differs | `argus-core::js` and `argus-board` writers already proven by the server ports; layer 3 compares bytes; corpus lines with non-ASCII text and emoji at each limit |
| 3. Spec drift and mixed versions. coord keeps changing; rendered copies differ per checkout; Node and Rust write the same board during phase B | A port that is right for an old commit; one verb behaving two ways | The freeze or the ship-with-port rule; the matrix pinned to an `ai-config` commit; the engine switch only after a full render; the shared lock protocol already ported (`coord.store`) |
| 4. A hook that fails or hangs | Every prompt of every session slowed or broken | `catch_unwind`, exit 0 on any error, the 5 s lock wait under the 10 s timeout, hooks switched last |
| 5. Double effects in the fallback | A write applied twice | Exit 120 only before any read of stdin or any file; any other code is final |
| 6. Argument quirks | A typed command parsed differently | The hand port of `parseArgs` with its own cases, and every corpus line goes through it |
| 7. Autostart after the switch-over | Two servers or none at session start | `hook.autostart` ported, and the Rust hook starts its own exe with the Node guards (pid, `quit.flag`, `start.lock`, home overrides) |

## Open questions

| For | Question | Recommended default |
|---|---|---|
| Overlord | Land `forward.mjs` in `coord.mjs` with the engine off, under the contract of section 4? | Yes, now, so layer 2 can run |
| Overlord | Freeze coord features while WP2 to WP6 run, or ship each change with its port? | Freeze; fixes still land in Node with a corpus line |
| Overlord | Phase D: let `hooks.json` call `argus.exe coord` directly? | Yes, once phase C has run a week and the bench shows the gain |
| Lead | `argus-coord` depends on the agents, launchers, worktrees and notify crates for their pure rules? | Yes, so each rule exists once |
| Lead | `ureq` for the CLI client in place of `reqwest`? | Yes: blocking, no runtime start on the hook path |
| Lead | Skip a hook save that changes nothing? | Not before phase C; it changes when the dashboard sees a board move |
