<!-- @layer docs @kind doc -->
# Wake policy

A wake is a model turn on a thread, and a lead turn runs at a large context. So a thread stays
idle until Argus signals a reason on the board that needs that thread to act. Everything else is
a notice: the board shows it and the thread reads a one-line digest with its next prompt.

The evidence is the orchestration audit of 2026-11-01: about 70 percent of the wakes of a day
were system alerts (wake faults after restarts, process alerts about an emulator, stale
worktrees and items, the remote sync reminder), and lead to lead notes were half of all wakes.

The code is `crates/argus-wake/src/wake_policy.rs` (the table and the settings),
`crates/argus-wake/src/engine/policy.rs` (where the engine applies it),
`crates/argus-wake/src/notices.rs` (the digest), the prompt hook in
`crates/argus-coord/src/verbs/hook/notices.rs` and the board header in
`apps/web/src/screens/board/BoardHeader`.

## One table, one place

`WAKE_CLASSES` gives every reason of `WAKE_REASONS` a class, wake or passive. The engine reads
it in `WakeEngine::submit`, the one place every wake is queued. So every producer obeys it with
no change of its own: the owner post policy, the registry faults, the item, agent and sync
reminders, the worktree hygiene, the launcher wakes, the reaper, the process monitor and the
messages from outside (`POST /api/p/:key/wakes`). A reason the table does not list wakes, so
nothing new is ever lost.

A thread is woken for:

- an owner post, answer or decision for it (`owner-*`, `item-assigned`, `resume`, and
  `portfolio-*` sent down by the Overlord);
- a message to it that needs action (`thread-message`, and `lead-message` for a lead's decision);
- one of its agents finished or is gone (`agent-idle`, `agent-orphan`);
- its own test or run result (`test-*`, `bundle-complete`, `run-failed`);
- a real fault of its own work (`process-reaped`, a worktree removal it asked for that was
  refused, which comes as a `thread-message`).

It is never woken for, and these stay notices:

- `wake-fault`: `board-closed`, `no-listener`, `no-reaction` (after a restart too),
  `not-listening` and `delivery-error`, told to the Overlord;
- `worktree-review`, `worktree-stale`, `launcher-stale`, `launcher-master-missing`;
- `kanban-stale` (item staleness), `agent-stale` and `agent-checkin` (quiet agent reminders);
- `remote-sync-due`;
- `process-alert` (hogs and orphans, mostly not project work);
- `lead-info`: a lead's note to another lead (`coord lead <key> "<text>"`, kind note). A lead's
  decision (`--kind decision`) asks for action and stays `lead-message`.

The table itself is in `docs/architecture.md` under the wake engine.

## A passive row

A passive reason is still recorded: the row is written and closed at once with status
`passive`, so it is in the `wakes` mirror and still counts for the producers that space their
reminders by the last row (`last_at`). It opens no window, reaches no listener, is never
replayed to a late one and never starts the registry's reaction clock. A passive row is stored
as `coalesced` with no event to fold into, so the table keeps the three status words the Node
schema allows.

Rows queued before this policy with a passive reason are not replayed, and boot recovery files
them as notices.

## The debounce and the digest

On top of the table, a thread is woken at most once per `wakePolicy.debounceMs` (5 minutes).
The first wake after a quiet spell goes on the usual coalescing window (`wake.windowMs`). A wake
that comes inside the debounce is held: it opens one digest window that closes
`wakePolicy.digestMs` (20 minutes) after the last wake, and every later wake joins it, so the
thread gets one event with every line. An owner reason in a held window brings it forward to the
end of the debounce: the owner never waits 20 minutes.

The two settings live in `config.json`:

```json
{ "wakePolicy": { "debounceMs": 300000, "digestMs": 1200000 } }
```

`debounceMs: 0` turns the debounce off. The debounce is per project and thread, kept in memory:
a restart starts every thread free.

## Where the notices show

- The board header shows a `notices` count when the last 50 rows of the project hold passive
  ones; a click lists them, newest first, with their reason, thread, age and text.
- The prompt hook (`coord hook prompt`) of a registered thread asks
  `GET /api/p/:key/wakes/notices?thread=<id>&since=<lastSeen>` and prints one line, for example
  `Board notices since your last turn: 3 (kanban-stale 2, worktree-review 1). Newest: "...".
  Not wakes: the board header lists them.` A lead asks for every thread (`*`). The route looks
  back at most a day. A dashboard that is down or slow (1.5 s) prints nothing.

The note length limits are unchanged, and the SessionStart text still tells each thread to open
its board tab.
