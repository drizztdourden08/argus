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

Brock's local CI runner runs the checks (the same commands as CI) and writes one file per run,
atomically (temp file, then rename): `~/.coord/ci/<project>/<runId>.json`. Argus watches the
folder, indexes the runs into its database and shows them. No push to Argus is needed.

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
stream like the other collections (`c=prs`) and only reads it. The block that says where a record
stands on the way to GitHub is `replay` (older writers said `remote`; both are read):

```json
{
  "id": "PR-3", "title": "", "body": "", "head": "feat/x", "base": "main",
  "headCommit": "<sha>", "status": "open | merged | closed", "item": "I-12",
  "createdBy": "coordinator", "createdAt": "",
  "mergedAt": null, "mergeCommit": null, "mergedBy": null,
  "replay": { "state": "pending | done | failed", "at": null, "url": null }
}
```

`coord pr open|merge|close|list` write it. A merge is a local git merge into `base` done by the
tooling (Brock's), then the record. Merge refuses while the latest check run of `headCommit` has a required step not passing,
the way branch protection does.

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

All JSON, read only, on the main listener and on every door. The pages and the feed share one
server, so the feed urls in the answers are paths (`/releases/...`) that work on whatever host the
page was opened from.

| Route | Answer |
|---|---|
| `GET /api/remote` | `{ mode, replay: { prs, releases, packages, checks } }`: the switch and the counts of records whose `replay.state` is `pending`. `prs` counts the `prs` collection of every board, `packages` the builds of the registry index that carry a pending `replay` (0 when the index has none). |
| `GET /api/releases` | `{ projects: [{ project, channels: [{ channel, latest, versions }] }] }` |
| `GET /api/p/<key>/releases` | `{ channels: [{ channel, latest, versions }] }` |
| `GET /api/p/<key>/checks?commit=<sha>&pr=<PR-n>` | `{ runs: [{ id: "C-n", runId, ...the record }] }`, newest first. `commit` matches a prefix of at least 7 characters either way, `pr` the record's `pr`. |
| `GET /api/p/<key>/checks/<C-n>/log?step=<i>` | the log text of a step (`text/plain`, the last 4 MiB, `X-Log-Truncated: start` when cut) |
| `GET /api/registry` | `{ builds, consumers, scopeRedirected, consumersNote }` |
| `GET /releases/<project>/<channel>/<path>` | the update feed (below) |

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
merged only by ancestry or `git cherry`.

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
