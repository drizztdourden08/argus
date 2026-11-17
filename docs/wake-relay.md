<!-- @layer docs @kind doc -->
# Wake relay

The owner's rule (D-107): no tokens are spent to keep a thread awake. A wake is paid for only when there is
something to deliver. The desktop app pauses a session even with Remote Control on, so a paused thread is woken on
demand.

## The flow

1. A thread is due: a queued wake, no listener, live on its board (`GET /api/wakes/due`). Reminders (`open-resend`)
   do not count: they stay queued for the thread's next resume but never make it due on their own.
2. After the wake has waited 30 seconds, Argus picks a relay: a thread with a v2 listener. The Overlord comes first
   when it is not in a turn, then any thread not in a turn, then a thread in a turn. Never the target itself.
3. Argus posts one inbox note from `Argus` to the relay. It names every due target of that relay at once, with the
   desktop session id of each. Then it wakes the relay with the reason `wake-relay`, which skips the coalescing
   window and the debounce, so the relay hears it at once.
4. The relay sends `WAKE <thread>: run coord inbox (relayed by Argus)` to each target with the desktop's
   send-to-session tool. The target resumes and its listener takes the queued wake.
5. Still due 2 minutes after a note? Argus sends one more, to a different relay when there is one.
6. No relay, or still due after two notes: Argus resumes the target itself, headless.
7. Still due 10 minutes after that: one owner notification, then nothing more for that target for 1 hour.
8. A target is forgotten the moment it is no longer due.

## Headless resume

The CLI's print mode: `-p --resume <cliSessionId> "<wake text>" --output-format json`, hidden, in the session's folder, from the
desktop's own CLI. The env drops the auth variables, the session variables of the desktop app and the OAuth switches (`strip_env` in
`argus-overlord`), so the CLI uses its own sign-in. It is skipped when a running CLI process carries that
session id. It is killed by pid after 20 minutes, never by name. The session id and folder come from the desktop's
session record (`cliSessionId`, `cwd`).

## Config

`wakeRelay.enabled` and `wakeRelay.headless`, both true by default, in config.json.

## Where the code is

- `crates/argus-wake/src/relay`: the picker and the attempt tracker, pure and tested.
- `crates/argus-wake/src/service/relay.rs`: the pass that runs on each registry tick.
- `crates/argus-server/src/adapters/wake`: the note, the headless resume and the owner notice.
- The `relay` field of each row of `GET /api/wakes/due` is the last attempt (`kind`, `at`, `via`) or null.

The relay stays off in a shadow run and in a server without the accounts service.
