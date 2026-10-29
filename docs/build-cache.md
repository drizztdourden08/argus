<!-- @layer docs @kind doc -->
# The build cache

Brock keeps one cache per user of what a fresh worktree would otherwise build or download again:
each `@drizztdourden08` package version compiled for a target, and each Electron binary. Brock's
`docs/build-cache.md` is the contract (the folder layout, `manifest.json`, how entries are written
and read). Brock and Tessera only add entries and record when one was used. They never delete one.
Argus owns usage tracking and pruning. This page is Argus's side.

## The rule

- The cache root is `BROCK_BUILD_CACHE_DIR` when it is set, else `~/.brock/cache/build-cache` (on
  Windows `%USERPROFILE%\.brock\cache\build-cache`). A relative root is refused.
- An entry (one `<buildKey>` folder) is in use when any known project or git worktree resolves
  that name at that exact version in its lockfile: `pnpm-lock.yaml` or
  `node_modules/.pnpm/lock.yaml` at its root. For Electron that is the `electron` version. The
  lockfile reader is small and reads only the `packages:` keys, for lockfile versions 5, 6 and 9.
- Known means every project root (the config `projectRoots` and the roots of the registered
  boards) and every git worktree the Worktrees scanner found. A leftover folder, a folder being
  deleted, a gone worktree and a folder that is not a repository are not users.
- An entry that is not in use may be deleted once `lastUsedAt` is older than the grace period. An
  entry whose manifest cannot be read is judged by the time of its folder.
- A folder under `.tmp/` last touched more than a day ago is left over from a build that was cut
  off, and is deleted.
- Only whole `<buildKey>` folders are deleted. A `<version>` or `<name>` folder is removed once it
  holds no entry.

## How a prune stays safe

- **Whole folders.** A `<buildKey>` folder is renamed into `.tmp/` in one step, then deleted. A
  reader never meets half an entry. If the delete is cut short, what is left sits in `.tmp/` and
  the next prune removes it.
- **Junction safe.** The delete is the one the Worktrees tab uses (`Fs::remove_tree` of
  `argus-worktrees`): a link is unlinked, never followed.
- **Inside the root only.** Every path is checked against the cache root before it is touched.
  A path outside it, with a `.` or `..` segment, a link, or a root that is a drive or `/` is
  refused and listed in the result (`refused`). A segment must match `[A-Za-z0-9_.+@-]+`.
- **Unknown users stop the prune.** If a lockfile of a known project or worktree exists but cannot
  be read, nothing is deleted and the page says which file.
- **Worktrees first.** No prune runs before the Worktrees scanner has looked at every project
  once, since a worktree that is not known yet may use an entry.
- **Never in shadow mode.** A shadow server lists the cache and deletes nothing; the prune route
  answers `read_only`.

## When it runs

- Once after the start (`buildCache.startDelayMs`, default 60 seconds, then as soon as every
  project's worktrees were scanned), then every 24 hours.
- On demand: `POST /api/cache/prune`, and the Prune now button of the Cache tab.
- Every on-demand prune leaves one activity line (kind `build-cache`) with the counts and the
  bytes freed, for example `build cache pruned: 3 entries, 1 temp folder, 12.4 MB freed`. A
  scheduled prune leaves one only when it deleted, refused or failed something.

## Config

`buildCache` in the dashboard's `config.json`:

| Key | Default | Meaning |
|---|---|---|
| `graceDays` | 14 | Days an unused entry is kept after its last use. A whole number of at least 1. |
| `startDelayMs` | 60000 | How long after the start the first prune waits. |

The grace period is read on every call, so a change applies to the next prune without a restart.

## Routes

- `GET /api/cache` answers `{root, exists, graceDays, rows, totals, problems, lastPrune, shadow}`.
  A row is one cached version: `kind` (`package` or `electron`), `name`, `version`, `buildKeys`,
  `users` (`{label, kind}` with `kind` `project` or `worktree`), `userCount`, `sizeBytes`,
  `lastUsedAt`, `inUse`, `deleteAfter` (unused rows), `prunable` and `prunableBytes` (what a
  prune would delete now). `totals` has the versions, entries, sizes (total, in use, unused,
  ready to delete) and the left-over `.tmp/` folders. `lastPrune` is the last prune of this run,
  or `null`.
- `POST /api/cache/prune` prunes now and answers `{at, deleted, abandonedDeleted, freedBytes, refused,
  failed}`. It answers 403 `read_only` in shadow mode and 409 `conflict` while the worktrees were
  not scanned yet or a lockfile could not be read (nothing is deleted then).

## The Cache tab

The overview has a Cache tab beside Processes, Activity and Notifications. It shows the totals,
then one row per cached version: kind, name, version, build keys, every project and worktree that
uses it (folded when long), the count of users, the size and the last use. An unused version says
`unused, deleted after <date>`. Prune now asks first and lists what will go.

## Tests

- `crates/argus-cache/tests/prune.rs` runs the rule on temporary cache roots and temporary
  projects with small lockfiles: in use, unused inside the grace period, unused past it, old and
  young `.tmp/` folders, an unreadable lockfile. The unit tests cover the lockfile formats and a
  path outside the root being refused.
- `crates/argus-server/tests/cache.rs` covers the routes, the activity line, the grace period of
  the config, shadow mode, the wait for the worktree scan and a worktree lockfile.
- `apps/web/tests/overview/cache.tab.test.tsx` covers the tab.
- No test touches the real `%USERPROFILE%\.brock`: the test server turns the cache service off
  unless a test gives it a temporary root.
