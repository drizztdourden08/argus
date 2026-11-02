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
  refused, which comes as a `thread-message`);
- a dependency on another board that needs it to act: a request for it (`dep-request`), or the
  request it made is done, dropped, gone or blocked (`dep-resolved`, `dep-blocked`).

It is never woken for, and these stay notices:

- `wake-fault`: the record of a fault (`board-closed`, `no-listener`, `no-reaction` (after a restart
  too), `not-listening` and `delivery-error`) for the Overlord. The Overlord is woken for the
  fault itself by `fault-open` (see Open faults and notices);
- `worktree-review`, `worktree-stale`, `launcher-stale`, `launcher-master-missing`;
- `kanban-stale` (item staleness), `agent-stale` and `agent-checkin` (quiet agent reminders);
- `remote-sync-due`;
- `process-alert` (hogs and orphans, mostly not project work);
- `lead-info`: a lead's note to another lead (`coord lead <key> "<text>"`, kind note). A lead's
  decision (`--kind decision`) asks for action and stays `lead-message`;
- `dep-cancelled`, `dep-reopened`: a request was withdrawn, or is open again.

## Dependencies

A request from one board to another (`coord dep`) is a record in the ledger on the portfolio
board. Each transition adds a history row `D-n#seq`, and `crates/argus-wake/src/dep_wakes` turns
each row into at most one wake. The subject is `D-n#seq`; the wakes table is asked for it, so a
restart never sends a row twice. The target is the thread of the item on that side while the
thread is live, else the lead of that project. A paused thread hears it once it resumes.

| Reason | Class | Immediate | Who | When | Text |
|---|---|---|---|---|---|
| `dep-request` | wake | no | the assignee thread, else its lead | the request reaches `open` | `<p>/<I-7> asks you for <I-12> "<title>" (<D-n>)` |
| `dep-resolved` | wake | yes | the requester thread, else its lead | the assignee item is `done`, `dropped` or `gone` | `<p>/<I-12> is <state> (<D-n>): resume <I-7>` |
| `dep-blocked` | wake | yes | the requester thread, else its lead | the assignee blocks on the requester | `<p>/<I-12> is blocked on you (<D-n>): <reason>` |
| `dep-cancelled` | passive | no | the assignee thread | the requester cancels | `<p>/<I-7> cancelled its request <I-12> (<D-n>)` |
| `dep-reopened` | passive | no | the requester thread | `blocked` or `done` goes back to `open` | `<p>/<I-12> is open again (<D-n>)` |

Here `<p>/<I-7>` is the requester item and `<p>/<I-12>` the assignee item, each with its own
project. Immediate means the window closes at once, and in the debounce the wake goes at the end
of the debounce.

`kanban-stale` skips an item that waits on a request that is `opening` or `open`, and an item
whose `requestedBy` request is `blocked`: the reminder would only nag a thread that cannot act.

The table itself is in `docs/architecture.md` under the wake engine.

## Open faults and notices

A passive notice raised once and never read left faults open for 20 hours. So what stays open
is pushed (`crates/argus-wake/src/reminders`, driven by `WakeService::reminder_tick`, which the
service tick runs after each registry tick):

| Reason | Class | Immediate | Who | When | Text |
|---|---|---|---|---|---|
| `fault-open` | wake | yes | the Overlord | a wake fault opens (not `board-closed`, which is the `wake-fault` notice only) | `<p>/<thread>: <reason> (<detail>)` |
| `open-resend` | wake | no | the Overlord | every `reminders.resendMs` (15 min) while a fault, a `worktree-stale` or a `remote-sync-due` notice is open | `2 faults open, oldest 3h; 1 worktree-stale open` |

The line is ONE wake for everything open; the age is the shared age rule (`age_text`). The
Overlord's own fault is left out of it. The clock starts when something is first seen open and
stops when nothing is.

- After `reminders.ownerMs` (1 h) a fault also reaches the owner, once, as a toast and a
  notification, through the owner sink the server plugs in.
- `board-closed` is raised only for a thread that is working (inside a turn); an idle thread is
  never faulted, and the fault clears when the thread goes idle or its board opens.
- A fault clears by itself when its cause is gone, and the activity log says why: the thread
  listens again (`not-listening`, `no-listener`), it shows a sign of life after the fault opened
  (`no-reaction`, `delivery-error`), its board opens or it goes idle (`board-closed`).
- `remote-sync-due` is open while the lead's sync is overdue (a record ends it); `worktree-stale`
  comes from a notice source the server adds (`WakeService::add_notice_source`).

```json
{ "reminders": { "resendMs": 900000, "ownerMs": 3600000 } }
```

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
