# Remote sync tracking

Owner order, 2026-11-01 (board item I-41). Each lead runs one full remote sync a day: push,
pull requests, packages, releases and deploys. The lead then records it, Argus shows how old the
last one is, and wakes the lead when a day has passed without one.

The record side lives in the Node `coord` (`shared/tools/coord/remote-mode.mjs`, commit efa8784
of the config repo) and in the Rust `coord` (`crates/argus-coord/src/verbs/remote/mode.rs`). Both write
the same file.

## The record

```text
coord remote synced [--note "<what went out>"] [--project <key>] [--as <name>]
```

It writes `syncs.<project>` in `~/.coord/remote.json` and keeps the mode and the `ciRoots`:

```json
{ "mode": "local", "since": "", "by": null,
  "syncs": { "argus": { "at": "2026-11-01T10:00:00.000Z", "by": "argus", "note": "pushed 2 PRs" } } }
```

- The project is the one `coord` finds for the caller (`--project`, `COORD_PROJECT`, the session,
  the checkout). `by` is `--as`, else the project key. The note is cut to 400 characters.
- Output: `remote sync of <project> recorded at <iso>[: <note>]. Due again in 24 h.`
- `coord remote status` adds one line per project after the mode:
  `last remote sync per lead:` then one line per project, indented, `<project>: <iso> (<note>)`, or `no remote sync recorded
  yet.` when there is none. A mode switch and `ci-root` keep the records.
- Code: `argus_core::remote_state::record_sync` writes, `argus_core::remote::{syncs, last_sync,
  SyncRecord}` read.

## What Argus shows

`GET /api/remote` carries `syncs`: project key to `{ at, by, note }`, empty when there is none.
The pages poll it every 60 seconds (the same poll as the local mode banner) and show, with the
shared Age component:

- on every board header: `last remote sync 3h ago`;
- on every overview project card that has a board: the same line under the state row.

The tone follows the age: normal while fresh, amber (Tessera warning) from 20 hours, red
(danger) past 24 hours, and `never` (normal tone) before the first record. The tip of the line
names who recorded it and the note. The tones live in
`apps/web/src/shell/SyncStamp/behavior/sync-tone.ts`.

## The wake

`argus_wake::sync_wakes` (a producer of the wake service, next to the item and agent wakes).
Reason `remote-sync-due`; it is not an immediate reason, so it coalesces like the Kanban
reminders.

- A project is checked when its board has a coordinator thread that is registered and not ended
  (not `done`, not `archived`, not a placeholder). The Overlord's scope (`portfolio`) and the
  portfolio board are left out.
- The sync is due when `due_ms` (24 hours) have passed since the last record. A project that
  never synced counts from the day its lead registered (`registeredAt` of the thread), else from
  the start of Argus.
- The lead gets ONE wake when it becomes due, and the same again every `repeat_ms` (24 hours)
  while it is still due. The last wake is asked of the wakes table (as the Kanban reminders do),
  so a restart does not send it again at once. A new record moves the due time forward.
- Text, exactly: `remote sync due: last <ago>, run the full sync, then coord remote synced`, with
  `<ago>` the short age of the board (`25h ago`, `2d ago`) or `never`.
- The check runs on the wake service's timer, every `tickMs` (60 seconds). The time comes from
  the injected clock, so a test drives it with a manual clock.

Config (`config.json`, section `remoteSync`, every value in milliseconds, a bad value keeps the
default): `tickMs` 60000, `dueMs` 86400000, `repeatMs` 86400000.

## Activity log

Two kinds, one line each:

- `remote-sync`: `<project>: remote sync recorded by <by>: <note>`, when a record newer than the
  one seen before appears. A record older than the start of Argus is not news, so a restart does
  not log it again.
- `remote-sync-due`: `<project>/<lead>: remote sync due, last <ago>`, when the wake is sent.

## Tests

- `crates/argus-wake/tests/it/sync_wakes.rs`: never synced, 19 h, 20 h, 24 h once, 48 h again, an
  archived lead skipped, a new record clears it, the restart rule, the activity lines, the config
  and the service tick, all on a manual clock.
- `crates/argus-coord/tests/it/remote_switch.rs`: the verb, the status lines and the file.
- `crates/argus-core/src/remote.rs`: the records read back.
- `crates/argus-server/tests/it/records.rs`: `syncs` in `GET /api/remote`.
- `apps/web/tests/remote/sync-stamp.test.tsx`: the tones, the board header and the cards.
