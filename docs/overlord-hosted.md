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
