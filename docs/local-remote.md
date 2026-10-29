# Local remote: the Argus side (proposal)

Status: proposal for the joint plan of the Overlord, the Brock lead and the Argus lead. No code
yet. Owner order, 2026-10-29: development stays local, GitHub is used only on the owner's go, and
everything that needs the remote must still work as if it were there.

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

Checks run on the Argus run engine, not ad hoc: one run definition per check name and project
(for example `check-lint`, `check-test`, `check-build`), so they get the same hidden process tree,
log capture, stop, history and verdict as Play. The command list per project comes from the same
place CI reads it today (Brock owns that: the gate scripts in `brock.workspace.mjs`).

Record, kept in the dashboard database (not on the board: there are many), its log as a run log:

```json
{
  "id": "C-12", "project": "argus", "commit": "<sha>", "branch": "feat/x", "name": "test",
  "command": "pnpm test", "cwd": "X:/argus/.claude/worktrees/agent-x",
  "status": "queued | running | success | failure | cancelled",
  "exitCode": 0, "startedAt": "", "endedAt": "", "durationMs": 0,
  "run": "R-40", "by": "coordinator", "pr": "PR-3"
}
```

API: `POST /api/p/<key>/checks` `{commit, branch, name, cwd?, pr?}` queues one run and answers the
record; `GET /api/p/<key>/checks?commit=<sha>|pr=<PR-n>`; the record's log is the run's log. CLI:
`coord check <name>... [--commit <sha>] [--pr PR-n]`. A commit's status is the set of its latest
check per name, as GitHub shows it.

## Local pull requests

A board collection `prs` (project scoped, like items), ids `PR-n`:

```json
{
  "id": "PR-3", "title": "", "body": "", "head": "feat/x", "base": "main",
  "headCommit": "<sha>", "status": "open | merged | closed", "item": "I-12",
  "checks": ["C-12", "C-13"], "createdBy": "coordinator", "createdAt": "",
  "mergedAt": null, "mergeCommit": null, "mergedBy": null,
  "remote": { "state": "pending | done | skipped", "at": null, "url": null }
}
```

`coord pr open|merge|close|list` write it. A merge is a local git merge into `base` done by the
tooling (Brock's), then the record. Merge refuses while a required check of `headCommit` is not
`success`, the way branch protection does.

## Releases and the update feed

Store: `~/.coord/releases/<project>/<channel>/` (channel `latest`, `beta`):

```text
<channel>/
  latest.yml            electron-updater's file for Windows, written last
  latest-mac.yml        when built
  latest-linux.yml      when built
  <version>/
    manifest.json       Argus record (below)
    <files>             the installer, its .blockmap, archives
```

`latest.yml` uses relative `url: <version>/<file>` entries with `sha512` (base64) and `size`, the
format electron-updater's generic provider reads. Brock's release tooling writes a version folder
under `.tmp/` and renames it into place, then rewrites `latest.yml`. Argus only reads.

`manifest.json`:

```json
{
  "schema": 1, "project": "archipelia", "channel": "latest", "version": "1.4.0",
  "commit": "<sha>", "tag": "v1.4.0", "notes": "markdown", "createdAt": "", "by": "coordinator",
  "files": [{ "name": "Archipelia-Setup-1.4.0.exe", "size": 0, "sha512": "<base64>" }],
  "checks": ["C-20"], "remote": { "state": "pending", "at": null, "url": null }
}
```

Feed: Argus serves the store read only at `http://releases.localhost/<project>/<channel>/` on the
port 80 door (and at `http://127.0.0.1:47615/releases/<project>/<channel>/` as the fallback), with
Range requests for differential downloads and the right content types. In local mode an app is
built with electron-builder's `generic` provider pointing there; in remote mode it keeps its
GitHub provider. The feed lists the files that are there; nothing else.

## Packages

The Overlord's `localpkg` owns the registry. Argus reads its index (path and format from that
work) and shows it; each package record carries the same `remote` block for the replay.

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
with their assets. Each record's `remote` block moves to `done` with its URL; a failure stops the
replay at that record and says why.
