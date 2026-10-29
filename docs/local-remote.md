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

A board collection `prs` (project scoped, like items), ids `PR-n`:

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

Brock's release runner writes `~/.coord/releases/<project>/<version>/`: `release.json`,
`notes.md`, the package tarballs and the app assets under GitHub's asset names, including
electron-builder's own `latest.yml` (sha512). Written atomically; Argus only reads.

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

Feed: Argus serves `http://releases.localhost/<project>/` (and
`http://127.0.0.1:47615/releases/<project>/`) as the newest non-prerelease version's `latest.yml`
with each `url` rewritten to `<version>/<file>`, and `/beta/` the same over prereleases. The
version files are served under `<version>/`, with Range requests for differential downloads. In
local mode an app is built with electron-builder's `generic` provider pointing at that URL.

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
