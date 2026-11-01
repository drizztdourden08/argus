<!-- @layer docs @kind doc -->
# When the process monitor writes

The sampler runs every few seconds. It keeps the listing (collection `procs`) and the summary
(doc `meta/procs`) in the `_portfolio` scope. A write on every tick would churn SQLite and push a
stream frame to every open page, so the monitor writes only when a reader would see a change.

The code is `crates/argus-procs/src/publish.rs`. After each sample it builds a print of the
listing and the meta and compares it with the print of the last successful write. A match means
no store call at all.

## What does not count as a change

These fields move on every tick by nature, so they are left out of the comparison:

- `cpuPct`, `memMB`, `ageMin` and `idleMin` of a row: the page shows them, but a number that
  drifts is not news. Anything they cause that matters shows in `status` or `flags`, which are compared.
- `sampledAt`, `tookMs`, `totalMemMB` and each rule's `memMB` of the meta.

Everything else counts: a row added or gone, status, flags, the parent, the rule, the
protection, the per-rule counts, the rules, the platform and the error.

## The heartbeat

While samples keep coming, the store is written at least once every `HEARTBEAT_MS` (60 s of
sample time), so the volatile fields and "sampled at" stay fresh within a minute. A failed
sample never counts as a heartbeat; it writes only when its error is new.

## What is never held back

Alerts, the wake, the activity lines, kills and the reaper do not go through this gate. They act
when they happen. `republish` (after a hard resync wiped the scope) always writes. A write the
store refused is not recorded, so the next sample tries again.
