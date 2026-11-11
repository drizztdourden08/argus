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
- its own test or run result (`test-*`, `run-failed`; a run of its plan that ended is `plan-run-done`);
- a real fault of its own work (`process-reaped`, a worktree removal it asked for that was
  refused, which comes as a `thread-message`);
- a dependency on another board that needs it to act: a request for it (`dep-request`), or the
  request it made is done, dropped, gone or blocked (`dep-resolved`, `dep-blocked`);
- a worktree of its with nothing attached that still holds work: approve its removal or ask the
  Overlord for an exception (`worktree-approve`, once per worktree); for the Overlord, such an ask
  left unanswered for 30 minutes (`worktree-report`, once);
- a change to an item it works on that asks it to act: an item or a to-do given to it
  (`item-assigned`, `todo-assigned`), the last blocker of its item settled (`item-unblocked`), every
  child of its epic or story done (`children-done`), a run of its test plan ended (`plan-run-done`).

It is never woken for, and these stay notices:

- `wake-fault`: the record of a fault (`board-closed`, `no-listener`, `no-reaction` (after a restart
  too), `not-listening` and `delivery-error`) for the Overlord. The Overlord is woken for the
  fault itself by `fault-open` (see Open faults and notices);
- `worktree-stale`, `worktree-unlinked`, `launcher-stale`, `worktree-review` (no longer sent),
  `launcher-master-missing`;
- `kanban-stale` (item staleness), `agent-stale` and `agent-checkin` (quiet agent reminders),
  `agent-unassigned` (an agent with no item 10 minutes after it started, once);
- `remote-sync-due`;
- `process-alert` (hogs and orphans, mostly not project work);
- `lead-info`: a lead's note to another lead (`coord lead <key> "<text>"`, kind note). A lead's
  decision (`--kind decision`) asks for action and stays `lead-message`;
- `dep-cancelled`, `dep-reopened`: a request was withdrawn, or is open again;
- `item-unassigned`, `item-status`, `item-ask`, `todo-quiet`: see Item events.

## Dependencies

A request from one board to another (`coord dep`) is a record in the ledger on the portfolio
board. Each transition adds a history row `D-n#seq`, and `crates/argus-wake/src/dep_wakes` turns
each row into at most one wake. The subject is `D-n#seq`; the wakes table is asked for it, so a
restart never sends a row twice. The target is the thread of the item on that side while the
thread is live, else the lead of that project. A paused thread hears it once it resumes.

| Reason | Class | Immediate | Who | When | Text |
|---|---|---|---|---|---|
| `dep-request` | wake | no | the assignee thread, else its lead | the request reaches `open` | `dep <D-n>: <p>/<I-7> needs your <I-12> "<title>"` |
| `dep-resolved` | wake | yes | the requester thread, else its lead | the assignee item is `done`, `dropped` or `gone` | `<p>/<I-12> is <state> (<D-n>): resume <I-7>` |
| `dep-blocked` | wake | yes | the requester thread, else its lead | the assignee blocks on the requester | `<p>/<I-12> is blocked on you (<D-n>): <reason>` |
| `dep-cancelled` | passive | no | the assignee thread | the requester cancels | `<p>/<I-7> cancelled its request <I-12> (<D-n>)` |
| `dep-reopened` | passive | no | the requester thread | `blocked` or `done` goes back to `open` | `<p>/<I-12> is open again (<D-n>)` |

The hook adds `. Run: coord inbox`, so the thread hears `WAKE ... dep D-11: ... Run: coord inbox`.
Here `<p>/<I-7>` is the requester item and `<p>/<I-12>` the assignee item, each with its own
project. Immediate means the window closes at once, and in the debounce the wake goes at the end
of the debounce.

`kanban-stale` skips an item that waits on a request that is `opening` or `open`, and an item
whose `requestedBy` request is `blocked`: the reminder would only nag a thread that cannot act.
A test run that is `ready` and assigned to the owner (and the plan of one) counts as waiting on
the owner, so `kanban-stale` does not nag the lead about it.

The table itself is in `docs/architecture.md` under the wake engine.

## Item events

`crates/argus-wake/src/item_watch` compares each export of a project board with the one before,
through the pure `argus_board::item_events(before, after)`, so a write by a lead's CLI wakes the
thread it concerns as an owner post does. Each event is `<id>:<rev>:<kind>`; the wakes table is
asked for it (`last_at`), so a restart that replays an export sends nothing twice. The first export
of a project after a start only seeds, and a write made while Argus is down is not compared (Argus
is always on). The owner's own posts keep their path in `policy`: an `item-assigned` wake sent in
the last minute for the same item is not sent again. When an event changes nothing in the
`rev` of the record that makes it (a parent whose last child closed, an item whose question was
answered), the highest `rev` among the records that make it names it, so a second occurrence is
a new event.

| Event | Who | Reason | Class | Text |
|---|---|---|---|---|
| item given to a thread by someone else | the new thread | `item-assigned` | wake | `I-7 (title) was assigned to you by <who>` |
| item taken away from a thread | the old thread | `item-unassigned` | passive | `I-7 (title) was taken from you by <who>` |
| to-do given to a thread or to the lead of another project | that thread, or that project's lead | `todo-assigned` | wake | `to-do H-3 (text) was assigned to you by <who>` |
| stage or lane changed by someone other than the thread that works it | that thread (the accountable thread when the owner holds it) | `item-status` | passive | `I-7 (title) moved doing to review (by <who>)` |
| a question or decision by another thread on the item | the thread that works it | `item-ask` | passive | `<who> asked on I-7 (Q-4): <text>` |
| the last blocker of an open item settled | the assignee (the owner: a line in the activity log) | `item-unblocked` | wake | `I-7 (title) is unblocked: Q-4 settled` |
| every child of an epic or story is done or dropped | the parent's assignee (the owner: a line in the activity log) | `children-done` | wake | `every child of I-7 (title) is done or dropped (3 in all)` |
| a run of a test plan ended | the plan's thread | `plan-run-done` | wake | `I-9 (title): run 3: 4 pass, 1 fail` |
| a to-do of another lead that blocks an item, quiet for 24 h | that lead | `todo-quiet` | passive | `to-do H-3 (text) has been quiet for a day and holds I-7 on <p>` |
| a request to another board that the item waited on landed | as before | `dep-resolved` (from the ledger, see Dependencies) | wake | not sent again here |

None of them is immediate. A change by the thread that holds the item, or a thread giving an item to
itself, says nothing. Moving an item to another thread is the two assignment events, not also a status
change. Giving an item to the owner while the same thread stays accountable for it takes nothing away
from that thread.

Owner notices (a toast `ping` and a line in Notifications, by `auto`) go through the same owner sink
as the open faults, so they are told once (a mark in kv by topic, project, thread and event) and
obey the toast limits of `Notify::send` (3 toasts a minute per thread, 8 a minute for all, then
recorded only with one summary). Each has a switch under `notify.auto`:

| Event | Switch | Title | Text |
|---|---|---|---|
| item assigned to the owner by a thread | `assigned` | Assigned to you | `<p>: I-7 (title) was assigned to you by <who>` |
| to-do assigned to the owner (`assignee` `owner`) | `assigned` | To-do for you | `<p>: to-do H-3 (text) was given to you by <who>` |
| a bug the owner reported reaches `verifying` | `verify` | Ready to verify | `<p>: I-7 (title) is ready to verify` |
| a test run becomes `ready` on a live build, not handed to the owner | `run` | Ready to run | `<p>: test run I-9 (title) is ready to run on <build>` |
| a test run is handed: `ready`, assigned to the owner, build live | `handed` | Ready to test | `<Project>: <title> is ready to test (N tests)` (the notify handed ping, not the item watch) |

A handed run pings the owner once, through the notify handed ping (D-64). The item watch sends
neither the `run` notice nor the assigned notice for it; one check, `argus_board::handed`, says
which run is handed for both.

A to-do with no assignee is the owner's as before and pings nobody: only an explicit `owner` does.
The owner never hears of a change he made himself.

A to-do assigned to a thread or to the lead of a project does not count as needing the owner:
`refresh_needs_human` counts only pending to-dos with no assignee or the assignee `owner`, and so do
`needsYou` and `ownerItems` on the project cards. `ownerItems` lists `{id, title, type, stage}` for
the open items whose `assignee` is `owner` and the pending to-dos that wait for him, for the
"Assigned to you" list of the Notifications tab.

## Account handovers

The accounts service (I-135, `docs/accounts.md`) asks the working leads for a handover before it
switches the Desktop app to another account. The subject is the handover record `H-n`. A lead is
working when its thread is inside a turn or it has a live agent record; an idle lead is not asked.

| Reason | Class | Immediate | Who | When |
|---|---|---|---|---|
| `handover-estimate` | wake | yes | a working lead | the account nears its switch point: answer with `coord handover estimate` and keep working |
| `handover-request` | wake | yes | a working lead | the ask point: bring each agent to a safe point, write `handover.md`, run `coord handover write` |
| `handover-ready` | passive | no | the Overlord | a lead marked its handover ready |

`handover-estimate` and `handover-request` skip the wake debounce (`DEBOUNCE_BYPASS_REASONS` in
`crates/argus-wake/src/reasons.rs`, applied in `engine/policy.rs`): a lead woken a minute ago still
gets them at once, because the switch does not wait for a digest. They are also immediate, so the
window closes at once. Only the coordinator of a board (the Overlord on `portfolio`) answers with
`coord handover estimate`, `write` or `status`.

## Open faults and notices

A passive notice raised once and never read left faults open for 20 hours. So what stays open
is pushed (`crates/argus-wake/src/reminders`, driven by `WakeService::reminder_tick`, which the
service tick runs after each registry tick):

| Reason | Class | Immediate | Who | When | Text |
|---|---|---|---|---|---|
| `fault-open` | wake | yes | the Overlord | a wake fault opens (not `board-closed`, which is the `wake-fault` notice only) | `<p>/<thread>: <reason> (<detail>)` |
| `open-resend` | wake | no | the Overlord | every `reminders.resendMs` (15 min) while a fault, a `worktree-stale` or a `remote-sync-due` notice or an unacknowledged stall is open | `2 faults open, oldest 3h; 1 worktree-stale open` |

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
- `worktree-unlinked` is told once per worktree, to its thread, when no item names the worktree an
  hour after Argus first saw it (a main checkout is never flagged). Link it with `coord worktree
  link <id|path> I-n`. Nothing is removed for being unlinked, and the notice never wakes.
- `remote-sync-due` is open while the lead's sync is overdue (a record ends it); `worktree-stale`
  comes from a notice source the server adds (`WakeService::add_notice_source`).

```json
{ "reminders": { "resendMs": 900000, "ownerMs": 3600000 } }
```

## Unread wakes and stalls

A wake the thread skims past is not done. `crates/argus-wake/src/stalls` watches the wakes that ask
a thread to act (`dep-request`, `item-assigned`, `owner-answer`, `owner-note`, `owner-decision`,
`portfolio-decision`, `portfolio-note`; never a notice, a fault line, `lead-message`, `stall` or
`ack-nudge`) and the boards, once a minute (`stall.tickMs`, driven by `WakeService::stall_tick`).
A wake is acknowledged when the thread read its own inbox after it (`coord inbox`), when the
thing it asked for happened (the dependency was read or closed, or its item left `todo`; an
assigned item left `todo` or moved to another thread), or when the thread ended.

| Reason | Class | Immediate | Who | When | Text |
|---|---|---|---|---|---|
| `ack-nudge` | wake | yes | the thread that did not read | 5 min after an unacknowledged wake (`stall.nudgeMs`), once | `2nd: <the original text>` |
| `stall` | wake | yes | the Overlord | a stall is raised, and again every `stall.resendMs` (30 min) until it is acknowledged or clears | `stall: rotp did not read D-11 for 10 min (it is in a turn)` |

- At 10 min (`stall.unreadMs`) an unacknowledged wake becomes an `unread` stall for the Overlord; at
  30 min (`stall.ownerMs`) the owner also gets a notice. The Overlord's own board gets the nudge and
  the notice, never a stall about itself.
- The other stalls come from the boards alone: `unstarted` (a dependency open 30 min with its item
  still `todo`), `idle` (a live lead with no turn, agent or process for 10 min while an unblocked
  item is `todo` or `doing`) and `silent` (an item `doing` for 60 min with no board write and no
  agent). See `crates/argus-wake/src/stall_rules`.
- A stall is open until its condition clears; the activity log gets a `stall` line when it is
  raised and a `stall-cleared` line when it clears. The Overlord records `coord stall ack
  <project> --note "..."`: an acknowledged stall stops its resend and leaves the 15 minute
  `open-resend` line, and stays on the page until it clears. A new occurrence is a new stall.
- `stall` is never waited on by the registry, as `fault-open` is not: a fault about a stall line
  would loop.

```json
{ "stall": { "enabled": true, "tickMs": 60000, "nudgeMs": 300000, "unreadMs": 600000,
  "ownerMs": 1800000, "unstartedMs": 1800000, "idleMs": 600000, "silentMs": 3600000,
  "resendMs": 1800000, "horizonMs": 86400000, "procIdleMin": 10 } }
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
