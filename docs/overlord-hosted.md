# The hosted Overlord

Argus can run the Overlord as its own `claude` child process billed to an API key, off the owner's subscription.
The plan is `.claude/plans/i159-hosted-overlord.md` (item I-159).

## The key

- The owner pastes the key in Argus > Settings > System. Argus keeps it DPAPI-encrypted in
  `<home>/secrets/anthropic-key.bin`; the page only ever sees its last four characters.
- The key exists in plain text only in the child's env. It is never logged, printed or put in a global env.

## The child env

The child starts from a cleared env built by `build_env` (`crates/argus-overlord/src/session/env.rs`).
It drops every variable of the parent whose name starts with `ANTHROPIC_` or `CLAUDE`, plus
`USE_LOCAL_OAUTH`, `USE_STAGING_OAUTH` and `CONTROL_PLANE_API_KEY`, then adds the key, `ARGUS_HOSTED=1`
and `MCP_TOOL_TIMEOUT`.

Why the prefixes: a session of the desktop app sets `ANTHROPIC_BASE_URL` to its own local proxy, which signs every
call with the subscription. When Argus is started from a shell of that app, its env carries that variable. The first
probe kept it and got `401 OAuth access token is invalid` on both turns, at zero cost. The billing guard alone
could not catch it: `apiKeySource` still read `ANTHROPIC_API_KEY`.

## The probe

`argus overlord probe [--data-dir <home>]` starts one hidden child in an empty temp folder, sends two short turns
and writes `<home>/overlord/probe-<ts>.jsonl`, `probe-<ts>.stderr.log` and `probe-facts.txt`, with the key
redacted everywhere. Exit 0 is a pass, 2 is the billing guard refusing, 1 is an error or a timeout.

## Facts from the probe of 2026-11-14

- CLI 2.1.325, the one the Desktop app ships. Its help lists `--effort`, `--max-budget-usd`,
  `--permission-prompt-tool`, `--fork-session` and `--include-partial-messages`.
- The `init` event comes only after the first user message, not at start.
- `apiKeySource` is `ANTHROPIC_API_KEY`; the model is `claude-sonnet-5-5`.
- Turn 1: 2 input, 65 output, 25,333 cache-creation tokens, `total_cost_usd` 0.0640.
- Turn 2: 2 input, 3 output, 106 cache-creation, 25,333 cache-read tokens, `total_cost_usd` 0.0668.
- The probe marked the cost as not cumulative, but a cache-read turn costing more than the first turn suggests the
  figure is the running total of the session. Story 6 (spend) counts spend from the usage numbers and checks this
  against the console before trusting `total_cost_usd`.
- 97 tools; the only MCP server is the owner's user-level `music-studio`, since the probe runs in an empty folder.
  The real session runs in the Overlord's home and must load `coord-dashboard` there (story 5 checks it).
- About 25,000 tokens of system prompt and tools are cached on the first turn. A wake that reuses the session reads
  them from the cache.

## How wakes reach the child

While the child runs, the server attaches a `HostedSink` (`crates/argus-wake/src/hosted.rs`) to the wake engine for the project `portfolio`, thread `coordinator`, v 2. It formats each event with `argus_wake::line::wake_line`, the same function the `coord hook listen` hook calls, so the text is identical on both paths. The supervisor (`crates/argus-overlord/src/session/supervisor/`) owns an inbox. An idle child gets the first message at once. While a turn runs, messages are held, and after the `result` they go as one user message, the texts joined by a blank line. The `init` event comes only after the first message, so the billing guard runs on the first turn. When the child dies, the sink returns `Delivery::Gone`, the server detaches it and the engine keeps the wakes pending (`registry/pending.rs`). The next start attaches a new sink, which replays them. A turn written but not answered when the child died is written again to the next child. `coord hook listen` exits 0 at once under `ARGUS_HOSTED=1`. Nothing starts unless `config.overlord.mode` is `live`; the default is `off`.

## Approvals

The child starts with `--permission-prompt-tool mcp__argus-overlord__permission_prompt` and an `--mcp-config` file naming `http://127.0.0.1:<port>/mcp/overlord`. A tool the project's own allow rules do not cover reaches the owner; the rest runs as before.

- **The endpoint** (`crates/argus-server/src/routes/mcp_overlord.rs`) is separate from `/mcp`. It answers only a bearer made for each child start: random, never the API key, never the dashboard token. It lives in memory and in the temp mcp-config file (`argus-overlord-mcp-<id>.json` in the temp folder), which is deleted when the child stops. Any other call is 401. The route takes no gate credential, so the `/mcp` Origin rules do not apply to it, and the handler checks the bearer itself.
- **The tool** is `permission_prompt` with `tool_name`, `input` and `tool_use_id`. It records a row in `overlord_approvals (id, at, tool, input_summary, status, decided_at, decided_by)`, sends an urgent toast whose link is `/?tab=overlord&approval=<id>`, and holds the call.
- **The answer** is the tool's text content, a JSON string: `{"behavior":"allow","updatedInput":<input>}` or `{"behavior":"deny","message":"..."}`. A wait of `approvalTimeoutMin` ends in a denial with "the owner did not answer". `MCP_TOOL_TIMEOUT` in the child env is that wait plus a margin, so the CLI never gives up first.
- **The tab** reads `GET /api/overlord/approvals` (pending and recent, newest first) and answers with `POST /api/overlord/approvals/<id>` and `{decision: "allow" | "deny", message?}`. Both are for the owner's page only. A second answer is 409.
- **When the child stops** the bearer dies, the file is deleted and every pending approval expires. A start also marks rows left pending by an earlier run as expired.
- The pure parts (queue, timeout, reply shapes, bearer check, mcp-config text) are in `crates/argus-overlord/src/approvals/`. The gate that holds the calls is `crates/argus-server/src/adapters/overlord/gate.rs`.

## Owner chat

`POST /api/overlord/message` with `{text}` is for the owner's page only. No MCP tool or coord verb can post it, so no thread can speak as the owner. The text must not be empty and must not pass 20,000 characters (400). Argus writes it to the child as a user message that starts with `Owner (Argus Overlord tab):` and a space, so the Overlord can tell the owner from a wake. It goes through the same link as the wakes: held while a turn runs and sent after the `result`. With no child up the answer is 409, "the hosted Overlord is not running". The pure parts are in `crates/argus-overlord/src/session/owner.rs`; the route is `crates/argus-server/src/routes/overlord_message.rs`.

## On the board

- **The thread.** The session-start hook sets `host: argus` on the thread when the child env has `ARGUS_HOSTED=1`. A desktop thread has no `host` field, and a desktop session on a thread that was hosted takes the field off. The wake logic reads it as `ThreadView::is_hosted` (`crates/argus-wake/src/board.rs`).
- **The hook text.** A hosted session start says "Waking: hosted. Wakes come as user messages; no listener, no board tab." in place of the board-tab and listener text (`crates/argus-coord/src/verbs/hook/wake_line.rs`). The tab-closing guard skips a hosted thread.
- **Faults.** A hosted thread never gets `board-closed` or `not-listening`. The sink the server attaches while the child runs counts as its listener. When the sink is gone for the board grace while the thread was seen alive, the registry opens `hosted-down` (`crates/argus-wake/src/registry/hosted.rs`). It clears itself when the sink is back. The Overlord cannot resolve its own outage, so this fault reaches the owner at once as a toast, not after the hour that other faults wait (`reminders/owner.rs`).
- **Account switch.** The handover of I-135 skips a hosted thread, and the resume task does not message it as the Overlord chat.
- **Boot.** With `config.overlord.mode` `live`, Argus starts the child at boot. `config.overlord.autostart` set to false keeps it down; the default is true.

## Session routes

`GET /api/overlord/status` is for the owner's page only. It answers `{mode, status, pid, startedAt, sessionId}`: `mode` is `off`, `shadow` or `live`, and `status` is the supervisor state (`stopped`, `starting`, `idle`, `turn`, `paused-budget`, `waiting-approval` or `crashed`). With no supervisor it is `stopped` with nulls. The route is `crates/argus-server/src/routes/overlord_session.rs`; the running supervisor is kept in `services/overlord/running.rs`.

`POST /api/overlord/control` with `{action: "start" | "stop" | "restart"}` is for the owner's page only, with the strict origin check, and answers the status body. An unknown action is 400. It is 409 for a start when the mode is not `live` ("the Overlord mode is off; set overlord.mode to live in config.json"), for a start when a child already runs and for a stop when none runs. A restart stops, waits up to 15 seconds for the old supervisor task to end (409 if it does not), then starts. Actions run one at a time, so two children never exist at once.

`PUT /api/overlord/budget` with `{budgetUsd}` is for the owner's page only, with the strict origin check. The number must be finite, at least 0 and at most 10000 (400 otherwise). It saves `overlord.budgetUsd` in `config.json`, keeps every other `overlord` field, and answers with the body of `GET /api/overlord/spend`.

## Boot revival

The desktop Overlord messaged each lead with no wake listener after a machine boot. With the hosted Overlord (mode `live`) Argus does it (`crates/argus-server/src/services/overlord/boot_resume.rs`, rule in `boot_rule.rs`).

- **When.** Two minutes after Argus starts, so the leads that are alive can connect their listeners. Mode `off` or `shadow` does nothing. A machine with no boot time reading (not Windows) does nothing. The relay stays off in a shadow run and in a server without the accounts service.
- **Who.** Each project board's coordinator thread (not ended) that has no listener for its thread or for `*`. The Overlord's own board is skipped.
- **What.** One wake for the lead (reason `boot-revival`, text "Argus restarted after a reboot; you had no listener"), submitted through the wake hook. A lead with no listener is a due thread, so the wake relay of I-161 (`docs/wake-relay.md`) takes it: a live listener thread is asked once to send_message the lead, and when none is live or the notes fail, Argus resumes the lead headless, then tells the owner. No Desktop scheduled task is written, so it works while the desktop app runs.
- **Once per boot.** Each resumed lead is recorded as `<board>/<thread>` in the key `overlord.boot-resume` of the database, with the boot time. A second start of Argus in the same boot (boot times within a minute) adds nothing for a recorded lead. A new boot starts a new list.

## Shadow mode (I-172)

`config.overlord.mode` `shadow` starts a child too, a read-only copy that runs beside the desktop Overlord and never acts. The pure rules are in `crates/argus-overlord/src/shadow/`; the wiring is `crates/argus-server/src/services/overlord/shadow.rs`.

- **The session.** The first start runs `--resume <desktop cliSessionId> --fork-session`, so the shadow has the Overlord's context under a new id and the desktop file is never touched. The id is read from the `sessionId` of the live coordinator thread on the Overlord's board, mapped to the id the desktop app resumes now (`session_map`), else the thread's own. A restart resumes the shadow's own session without forking again. With no desktop session the start fails with a fault, never a guess.
- **The thread.** Argus sends the shadow one first message that tells it to run `coord thread register coordinator-hosted`. Register's default role is `thread`, so `overlord_only` refuses it (K4). The allow list lets that one verb through, and never with `coordinator` in its options.
- **Permissions.** The argv adds `--disallowedTools` for Edit, Write, NotebookEdit, PowerShell and the dashboard MCP tools that write, so no allow rule of the project lets them through. Every other ask comes to `/mcp/overlord`, where the shadow answers at once: allow for Read, Grep, Glob, the dashboard MCP read tools and a Bash `coord` read verb (list or show of item, thread, entry, test, agent, board and the like, and a bare `coord inbox`), deny for the rest. A command with `;`, `&`, `|`, `<`, `>`, a backtick, `$`, a parenthesis or a newline is never a read. No row, no toast: the owner is not asked. A Bash call that the project's own allow rules already cover never reaches the permission tool, so `coord` itself holds the line: see the next point.
- **The server-side guard.** The `coord` router (`crates/argus-coord/src/cli/shadow.rs`) runs before every verb but the hooks. When the session has `ARGUS_HOSTED=1` and its caller thread on the board is `coordinator-hosted`, any call that is not a read verb, `thread register coordinator-hosted` (never with `--role coordinator`), `thread note` or a bare `coord inbox` (with or without `--ack`) is refused with "the shadow Overlord only reads". The rule is `coord_words_allowed` in `argus-overlord`, shared with the allow list. Known limits: `thread note` is allowed whatever thread it names, and a shadow that has not registered yet has no caller thread, so it is not yet guarded.
- **The wake mirror.** While the child runs, a v1 listener (`HostedSink::mirror`) is attached to every thread of `portfolio`. It pushes to the shadow only the wakes of the thread `coordinator`, the same line text the Overlord gets, and returns `Sent` for the rest. A v1 listener is never counted as received, never replaces another listener and gets no replay, so the desktop delivery is unchanged. A wake that arrives while the shadow is down is not replayed to it.
- **The "would do" log.** `<home>/overlord/shadow-would-do.jsonl`, beside the transcripts, redacted. At each `result` the shadow writes `{kind: "would-do", at, wakes, said}`: the wakes it took since the last result and what it said. Each refused call is `{kind: "denied", at, tool, summary}`. `GET /api/overlord/shadow` (the owner's page only) answers `{entries}`, the last 200, oldest first, so the tab can show both side by side later. `GET /api/overlord/status` reports `mode: "shadow"`.

## Switch-over and rollback (I-173)

`POST /api/overlord/mode` with `{mode: "off" | "shadow" | "live"}` is for the owner's page only, with the strict origin check. It answers `{from, to, already}`. Another word is 400. The step rule is `crates/argus-overlord/src/mode.rs`; the effects are `crates/argus-server/src/services/overlord/switch.rs` and `switch_board.rs`.

- **The legal steps.** Off to shadow, shadow to off, shadow to live, live to off. The same mode is `already: true` and does nothing. Off to live and live to shadow are 409 with `details: {from, to}`: live always comes after a shadow, and leaving live always goes to off. One step runs at a time, under the same lock as `/api/overlord/control`.
- **Off to shadow and shadow to off.** The shadow child starts or stops as at boot. A start that fails puts `off` back and answers 409.
- **Shadow to live.** Argus moves an older `~/.coord/portfolio/handover.md` aside to `handover.prev.md`, then wakes the desktop Overlord (reason `overlord-switch`): it writes its handover to that file and ends its turn. Argus stops the shadow by pid and starts the live child, resuming the shadow's session, or forking the desktop session when the shadow has none (409 when neither has one). It does not wait for the handover. The live child's first message tells it to run `coord thread register coordinator --role coordinator`, to read the handover file when it is there, and to ignore an `overlord-switch` wake. The threads `coordinator` and `coordinator-hosted` are marked done; the live child's register takes `coordinator` back with its own session, and its hooks set `host: argus`. A live start that fails puts `shadow` back and starts the shadow again.
- **Live to off.** Argus stops the child by pid, marks `coordinator` done and writes `~/.coord/portfolio/handover.md` from the hosted side: the time, the hosted session id, the thread note, the open items of the board (id and title) and how to take the thread back. The owner sends one message in the desktop Overlord chat (a session cannot start itself); it reads the file and runs `coord thread register coordinator`. The desktop session is never touched.
- **One coordinator at a time (K4).** `coord thread register coordinator` on the Overlord's board reads `overlord.mode`. In mode `live` only the hosted child (`ARGUS_HOSTED=1`) may register it; a desktop session is told to roll back first. In mode `off` or `shadow` the hosted child is refused. The rule is `crates/argus-overlord/src/seat.rs`.
- **The desktop chat.** The owner archives the desktop Overlord chat himself after the switch, and keeps it, never deleted, until a week of live use has passed.
