# Local remote: the Argus side

The agreed plan of the Overlord, the Brock lead and the Argus lead is
`X:/ai-config/docs/local-remote.md`; where this file differs, the plan wins. Status: the server
side is built (the switch, the release store and update feed, the check runs, the registry, the
`prs` collection; see the API section at the end). The pages are built by the web side. Owner
order, 2026-10-29: development stays local, GitHub is used only on the owner's go, and everything
that needs the remote must still work as if it were there.

## What Argus has today

- Board `builds` collection: bundle records (`tag`, `status` draft, ready, handed or closed,
  `title`, `note`, `launcher`, `thread`, times) and the board's `handedBuild`. The Builds tab shows
  bundles and launchers with Play, Stop and Log.
- Runs: launchers are run definitions executed by the Argus run engine (`argus-launchers`): hidden
  process tree, logs kept (30 per definition, 14 days), stop by tree, verdict, attach. Exposed as
  `coord run` and `/api/p/<key>/runs`.
- Proofs: test results with `--proof` images, and since I-36 question and decision images, stored
  under `<COORD_HOME>/<key>/proofs/` and served at `/files/<key>/proofs/<name>`.
- Build cache (I-37, live): Brock's per-user compiled package and Electron cache, read, shown on
  the overview Cache tab and pruned by Argus (`GET /api/cache`, `POST /api/cache/prune`).
- None of these exist yet: an artifact store, releases, check runs, local pull requests, an update
  feed, a package registry view.

## The switch

`~/.coord/remote.json`: `{ "schema": 1, "mode": "local" | "remote", "since", "by", "note" }`,
default `local` when the file is missing. Argus reads it on every request that needs it (no cache
beyond the file's mtime), shows a "local mode" banner with the counts of records waiting for
replay, and never calls GitHub while it says `local`. The one GitHub call Argus makes today
(`gh pr list` during a worktree removal with `--branch`) is skipped in local mode; the branch then
counts as merged only by ancestry or `git cherry`.

## Check runs (local CI)

Argus starts `brock ci` as a run on its run engine (logs, stop and history are Argus's), in a
detached worktree at the commit; the step list stays in Brock's standards. Brock's runner writes
one file per run, atomically (temp file, then rename): `~/.coord/ci/<project>/<runId>.json`. That
file is the source of truth: Argus watches the folder and indexes it as `C-n`, never a second
writer.

```json
{
  "kind": "check-run", "id": "", "project": "", "repo": "", "branch": "", "commit": "<sha>",
  "pr": "PR-3", "trigger": "push | pr | manual | release", "mode": "local",
  "status": "queued | running | pass | fail | cancelled | skipped-local",
  "startedAt": "", "finishedAt": "",
  "steps": [{ "name": "", "command": "", "cwd": "", "status": "", "startedAt": "", "exitCode": 0,
              "durationMs": 0, "log": "<path>", "required": true }],
  "runner": { "host": "", "os": "", "node": "", "pnpm": "" }, "summary": ""
}
```

A commit's status is its latest run; a step is green when its `status` is pass.

## Local pull requests

A board collection `prs` (project scoped, like items), ids `PR-n`. Argus exports it in the board
stream like the other collections (`c=prs`); the pages only read it. The block that says where a
record stands on the way to GitHub is `replay` (older writers said `remote`; both are read):

```json
{
  "id": "PR-3", "title": "", "body": "", "head": "feat/x", "base": "main",
  "headCommit": "<sha>", "status": "open | merged | closed", "item": "I-12",
  "checks": ["C-4", "C-5"],
  "createdBy": "lead", "createdAt": "",
  "mergedAt": null, "mergeCommit": null, "mergedBy": null,
  "replay": { "state": "pending | done | failed", "at": null, "url": null }
}
```

`coord pr open|merge|close|list` write it (the section "coord pr and coord check" below), and so
do the PR routes of the API. A merge is a local git merge into `base` done by the tooling
(Brock's), then the record. Merge refuses while the latest check run of `headCommit` has a
required step not passing, the way branch protection does; that check is Brock's, in
`brock pr merge --local`.

`checks` lists the `C-n` of the check runs whose `pr` is this record, oldest first. Argus keeps it:
after every scan of the check run folder it brings the list of each pull request that has runs in
line with the index (a merged or closed pull request keeps getting late runs). `closedAt` is set
when a pull request is closed without a merge.

The board ops behind it (`argus-board`, `apply_op`): `pr-open {set: {title, body?, head, base?,
headCommit?, item?}}`, `pr-update {target, set: {headCommit?, title?, body?}}`, `pr-merge {target,
set: {mergeCommit, mergedBy?}}`, `pr-close {target}` and `pr-checks {target, set: {checks}}`. Only
an open pull request is updated, merged or closed. A second open pull request for the same head and
base is refused. Each change but `pr-checks` leaves one audit line in the work log.

## Releases and the update feed

Brock's release runner writes `~/.coord/releases/<project>/<channel>/<version>/`: `release.json`,
`notes.md`, the package tarballs and the app assets under GitHub's asset names, and
electron-builder's own `latest.yml` (sha512), written last. Written atomically; Argus only reads.

```json
{
  "kind": "release", "project": "", "version": "", "tag": "", "commit": "", "title": "",
  "notes": "notes.md",
  "packages": [{ "name": "", "version": "", "tarball": "", "sha256": "", "size": 0 }],
  "assets": [{ "name": "", "path": "", "sha256": "", "size": 0, "kind": "" }],
  "publishedAt": "", "mode": "local",
  "replay": { "state": "pending | done | failed", "at": null, "url": null }
}
```

Feed: Argus serves `http://releases.localhost/<project>/<channel>/` (and
`http://127.0.0.1:47615/releases/<project>/<channel>/`). `<channel>/latest.yml` (and
`latest-mac.yml`, `latest-linux.yml`) is the newest non-prerelease version's file with each `url:`
and `path:` rewritten to `<version>/<file>` (a channel of prereleases only falls back to its
newest); a channel level file on disk is served instead when there is one. The version files are
served under `<version>/`, with Range requests for differential downloads. In local mode an app is
built with electron-builder's `generic` provider pointing at that URL.

## Packages

The Overlord's `localpkg` owns the registry. Argus reads its index (path and format from that
work) and shows it; each package record carries the same `replay` block.

## Pages

- Board tab **Changes**: local pull requests, each with its check dots (open a check to its run
  log), status, merge state and replay state.
- Board tab **Releases**: versions per channel, files, sizes, checks, which one `latest.yml`
  points at, replay state. Builds keeps bundles and launchers as today.
- Overview tabs **Releases** (every project), **Registry** (packages) and **Cache** (exists).
- Header banner in local mode: "Local mode: N pushes, N pull requests, N releases, N packages wait
  for the go."

## Replay on the owner's go

Order: push branches and `main` (merge commits included), then pull requests (opened on GitHub as
records of what was merged, closed with a link to the merge commit), then packages, then releases
with their assets. Each record's `replay` block moves to `done` with its URL; a failure stops the
replay at that record and says why.

## API

All JSON, on the main listener and on every door. The pages and the feed share one server, so the
feed urls in the answers are paths (`/releases/...`) that work on whatever host the page was opened
from. Everything is read only except the two write routes at the end of the table, which take the
same credential and Origin rules as the other writes (a page with its Origin, the CLI with the
bearer when the token is required) and answer `read_only` in shadow mode.

| Route | Answer |
|---|---|
| `GET /api/remote` | `{ mode, replay: { prs, releases, packages, checks }, ciRoots, syncs }`: the switch, the last full remote sync of each project (`syncs`, project key to `{ at, by, note }`, see `docs/remote-sync.md`) and the counts of records whose `replay.state` is `pending`. `prs` counts the `prs` collection of every board, `packages` the builds of the registry index that carry a pending `replay` (0 when the index has none). |
| `GET /api/releases` | `{ projects: [{ project, channels: [{ channel, latest, versions }] }] }` |
| `GET /api/p/<key>/releases` | `{ channels: [{ channel, latest, versions }] }` |
| `GET /api/p/<key>/checks?commit=<sha>&pr=<PR-n>&run=<R-n>` | `{ runs: [{ id: "C-n", runId, run, ...the record }] }`, newest first. `commit` matches a prefix of at least 7 characters either way, `pr` the record's `pr`, `run` the run of the run engine that made the record. `run` in each row is that `R-n`, or `null` for a record Argus did not start. |
| `GET /api/p/<key>/checks/<C-n>/log?step=<i>` | the log text of a step (`text/plain`, the last 4 MiB, `X-Log-Truncated: start` when cut) |
| `GET /api/registry` | `{ builds, consumers, scopeRedirected, consumersNote }` |
| `GET /releases/<project>/<channel>/<path>` | the update feed (below) |
| `POST /api/p/<key>/prs` | opens a local pull request. Body `{ title, body?, head, base?, headCommit?, item?, by? }`; `base` is `main` by default. Answers `{ pr, outcome, seq }`, `pr` being the record. 400 with the board's sentence for a bad field, a head equal to the base, an unknown item or a second open pull request for the same head and base; 404 for a project with no board. |
| `PATCH /api/p/<key>/prs/<PR-n>` | changes an open pull request. Body `{ action?, ... }`: `update` (the default) takes `headCommit`, `title`, `body`; `merge` takes `mergeCommit` (7 to 40 hex characters, required) and `mergedBy?`, and sets status `merged` and `mergedAt`; `close` sets status `closed`. `by?` names who acted (the page is always `human`; the default is `cli`). Answers as the POST; 404 for an unknown pull request, 400 for one that is not open. |
| `POST /api/p/<key>/checks` | starts a check run. Body `{ commit?, pr?, check?, job?, trigger?, localpkg? }` (below). Answers `{ run: "R-n" }`. |

A version is `{ version, tag, commit, title, publishedAt, notesUrl, assets: [{ name, size, kind,
url }], packages: [{ name, version, size, url, sha256, sha512 }], replay }`, newest first by semver.
`latest` is the version `latest.yml` points at (`null` when no version has one). `replay` is
`{ state, at, url }` or `null` when the record has none. `notesUrl` is `null` when the notes file is
missing.

A run is the check-run record of Brock with `id` set to `C-n`, `runId` the file name without
`.json` and `project` the folder. `C-n` is assigned per project in the order the runs were first
seen and kept in the dashboard database (`kv`: `check-run:<project>:<runId>` and the counter
`check-seq:<project>`), so a number never moves across a restart. A step log is served only when the
`log` path of the step is inside `~/.coord/ci` or inside the run's own `logDir`.

### Starting a check run

`POST /api/p/<key>/checks` runs `brock ci` as a run on the run engine of Argus: hidden (never a
window), in the root of the project (the root of `projectRoots` in `config.json`, else the one the
portfolio lists; 404 when none is known, 400 when it is not a folder), the logs kept like any run
(30 per definition, 14 days), Stop and the history are the run engine's (`POST
/api/p/<key>/runs/<R-n>/stop`, `GET /api/p/<key>/runs/<R-n>`). The run definition is made for the
request and is named `check`; it is not on the board. Up to 8 check runs of a project are live at
once (Brock queues them per repository); the time limit is 2 hours.

The command line is `brock ci [<check>] [--commit <sha>] [--pr <PR-n>] [--job <job>] [--trigger
<trigger>] [--localpkg] --project <key>`. Every part is optional but the project and is checked
to plain words before it reaches the shell: `check` letters, digits, `-` and `_` (40 at most, no leading `-`),
`commit` 7 to 40 hex characters, `pr` like `PR-3`, `job` one of `gate`, `upgrade`, `all`,
`trigger` one of `push`, `pr`, `manual`, `release`, `localpkg` true or false (true adds
`--localpkg`, so the run installs from the local registry). Anything else is a 400. `by?` names
who asked (the run's `by`; the page is `human`).

The program is `brock`; the `command` of the `checks` setting of `config.json` replaces it (typed
as in a shell, for a Brock that is not on the PATH, and for tests: a fake script).

The first line the run prints is the path of Brock's record (`~/.coord/ci/<project>/<runId>.json`).
When Argus reads it, it keeps the pair in the dashboard database (`check-link:<project>:<runId>`
holds `R-n`), so the checks answer shows `run` on that record, `?run=R-n` finds it, and the pair
survives a restart. A run that ends before it printed anything has no record and no link.

### coord pr and coord check

The exact command line, for the live Node `coord` to mirror. `coord pr` works on the board file
alone (it needs no running dashboard); `coord check` with a check run needs the dashboard (it is
started when it is down, and exit 3 says it would not come up, as for `coord run`). Every line
below is the whole of the output.

```text
coord pr open "<title>" --head <branch> [--base main] [--item <I-n>] [--body "<text>"] [--commit <sha>]
coord pr list [--status open|merged|closed] [--json]
coord pr merge <PR-n>
coord pr close <PR-n>
coord check [<check>] [--commit <sha>] [--pr <PR-n>] [--job gate|upgrade|all] [--trigger push|pr|manual|release] [--localpkg]
```

- `pr open` applies `pr-open` as the caller. The head commit is `--commit`, else what
  `git rev-parse --verify <head>^{commit}` says in the project root (empty when git cannot say).
  Output: `PR-3 opened: feat/x into main at 0123456` (the trailing `at <7 characters>` only when the commit is
  known). Refusals, exit 1 and `coord: <sentence>` on stderr: no title (`give the title: coord pr
  open "<title>" --head <branch>.`), no head (`--head <branch> is required.`), and the sentences of
  the board op (`head and base are the same branch.`, `PR-1 is already open for feat/x into
  main.`, `no item "I-9".`).
- `pr list` prints an aligned table `pr  status  branch  commit  checks  title`, one row per pull
  request, oldest first (`feat/x -> main`, the commit as 7 characters, the checks as `C-4 C-5`), or
  `no pull requests.`. `--json` prints an array of `{ id, title, body, head, base, headCommit,
  status, item, checks, mergeCommit, mergedAt, mergedBy, createdAt, closedAt, replay }`. This
  is what Brock's `brock pr merge --local` reads.
- `pr merge` refuses a pull request that is not open (`PR-3 is merged; only an open pull request
  merges.`) or unknown (`no pull request "PR-3".`). It runs `brock pr merge --local PR-3` in the
  project root (`--root <dir>`, else `projectRoots` of `config.json`, else the root the portfolio
  lists, else the git root of the folder it runs in), with the environment of the call. On exit 0
  the last line Brock printed must be the merge commit (7 to 40 hex characters): it is recorded
  (`pr-merge`, `mergedBy` the caller) and the output is `PR-3 merged into main at fedcba9`. On any
  other exit the pull request stays open and the exit is 1 with Brock's own words (stderr, else
  stdout) as `coord: <reason>`. A Brock that exits 0 without a commit is `brock merged but did not
  print a merge commit (it printed "<line>").`. On Windows Brock is started through `cmd /d /c`.
- `pr close` applies `pr-close`: `PR-3 closed`.
- `check` with a word or any of `--commit`, `--pr`, `--job`, `--trigger`, `--localpkg` posts to
  `POST /api/p/<project>/checks` with `by` the caller (`--localpkg` as `localpkg: true`; it is a
  bare flag, so the check word goes before it), then waits up to 30 seconds for the record and
  prints `R-5 C-9 check started`, or `R-5 check started (no record: <why>; coord run logs R-5
  shows the output)`. `coord check` with none of them is the board checker it always was
  (`coord check [--stale <minutes>] [--json]`).

`coord check` and `coord pr` appear in the usage text (two lines after `coord check [--stale]`).

### The registry

`/api/registry`: `builds` are the entries of `<localpkg home>/index.json` as `{ name, version, commit,
checkout, packedAt, tarball, size, dirty, replay }` (`commit` and `checkout` are `head` and `project`
in the tool's own index). `consumers` are the checkouts that use them, from `node localpkg.mjs status
--json`, run with a 5 s limit and kept 30 s: `{ path, since, hook, packages, on: true }`.
`scopeRedirected` is true when any consumer is on (the GitHub scope points at `127.0.0.1:9`). When
the tool is missing or too old to know `--json`, `consumers` is `[]` and `consumersNote` says why.
The `localpkg` home is `LOCALPKG_HOME`, else `~/.coord/localpkg`; `config.json` may set
`"localpkg": { "script": "...", "node": "..." }`.

### The switch

`~/.coord/remote.json` is read through `argus_core::remote` (`Switch`), one stat per call and read
again only when its time or size changed. A missing, unreadable or unknown file is `local`. In local
mode the one `gh pr list` of the worktree branch check (`pr_merged`) is skipped: a branch counts as
merged only by ancestry or `git cherry`. The same file also keeps `ciRoots` and the daily remote sync
record of each lead (`syncs`, written by `coord remote synced`, shown on the boards and the cards, and
watched by a wake after 24 hours without one): see `docs/remote-sync.md`.

### The update feed

- Two forms: `http://127.0.0.1:47615/releases/<project>/<channel>/<path>` and
  `http://releases.localhost/<project>/<channel>/<path>` on the localhost door (port 80). The Host
  `releases.localhost` is read as the `/releases` prefix. A project called `releases` cannot use
  that host for its board.
- `GET` and `HEAD` only (anything else is a 405). No credential is asked, so an app's updater works
  with `requireToken` on; the Host and peer checks still apply.
- Types: `.yml` `text/yaml`, `.exe` `.blockmap` and anything unknown `application/octet-stream`,
  `.tgz` `.gz` `application/gzip`, `.json`, `.md` and `.txt` as text. Never html or scripts.
- `Range: bytes=a-b` (also `a-` and `-n`) is a `206` with `Content-Range`; every answer for a file
  has `Accept-Ranges: bytes` and an `ETag`, and a stale `If-Range` gets the whole file. Several
  ranges or a header that does not parse get the whole file; a range past the end is a `416`.
- No directory listing. A path is split into plain names; `..`, `.`, an empty name, a back slash,
  a drive or stream colon, a trailing dot, a hidden or temp name (a `.tmp` or `.part` ending, `~`)
  and a link anywhere on the way are a 404, and the real path must stay inside the store.
- A release without a readable `release.json`, or in a folder with a temp name, is not in the index.

### Folders and watches

`~/.coord/releases` and `~/.coord/ci` are created when missing and watched (120 ms debounce, a
safety look every 30 s); the stores keep an index and parse a file again only when its time or size
changed.
