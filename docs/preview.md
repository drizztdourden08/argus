<!-- @layer docs @kind doc -->
# Preview mode

A preview lets the owner try a new build on a copy of the real boards, with writes, without
touching anything real. The shadow of `docs/switch-over.md` refuses every write, so it cannot
show the test runner, assignments or images. The sandbox of the same runbook writes, but its
services act on real folders and processes. A preview is the middle: board writes work, every
outward effect is off.

## What it does

`argus serve --preview --port 47617 --data-dir <preview>\dashboard`

- It accepts board writes: owner posts, test results, plan runs, item images, the dependency
  verbs and the `coord` verbs through the server.
- It turns every outward effect off, as shadow mode does. No worktree scan result is acted on and
  no worktree is removed. No process is killed (the reaper, the kill routes and the thread kill
  all answer `read_only`). No run starts, so Play is refused. No check run starts. No OS toast
  is shown and the tray balloon is not used. No remote call is made and the build cache is not
  pruned. Wakes are recorded and shown, never delivered.
- It never opens the localhost door on port 80 or the local names (DNS on port 53), whatever the
  copied config says.
- It writes `"preview": true` in the `config.json` of the preview home, like the shadow marker.
  A home with the marker is a preview for good: any later start on it (a verb, the tray, the
  hook autostart) is a preview whatever flags it got. To end a preview, delete its folder.
- `/api/health` says `"preview": true` and `"shadow": false`. A shadow still says `"shadow": true`.
- The port is 47617 when no `--port` and no `DASHBOARD_PORT` is given. The live dashboard keeps
  47615 and the shadow 47616.

## Accounts in a preview

The accounts feature is read-only in a preview and in shadow mode. A sync that applies, an undo, a Desktop switch (also "anyway"), saving a login and writing the accounts settings (shadow only; a preview home is a copy, so its settings may change) answer 403 `read_only`. Dry runs, compare, the GET routes, the handover route and a switch cancel stay open. The switching rules start no countdown, handover cycle or switch, nothing is written to the desktop app's scheduled tasks, the start-up repair of a cut-short switch does not run, and a login is never renewed. Usage is read from the Desktop log only. To see live usage in a preview, start it with `ARGUS_PREVIEW_ACCOUNTS_USAGE=1`; it only reads the usage endpoint with the logins as they are, and a shadow run ignores the variable.

## The refusal to start

A preview refuses to start, with a message, when `COORD_HOME` is the live `~/.coord` (the
default when the variable is unset), or when the dashboard home lies inside it. Nothing is
written before the refusal: no folder, no marker, no database. A preview home may be the
`dashboard` folder of its own copied `COORD_HOME`, which is the layout below.

## The steps for the lead

All hidden, with no window on the owner's desktop.

1. Build from the integration worktree: `pnpm build`, then `cargo build --release -p argus`.
2. Copy the binary, never the installed copy and never `target` directly:
   `target\release\argus.exe self-install --dir $env:TEMP\argus-preview\bin`.
3. Copy the data as the sandbox section of `docs/switch-over.md` does: every board folder of
   `~/.coord` except `dashboard` and `argus-shadow*`, into `$env:TEMP\argus-preview`. The
   database is copied with `sqlite3 -readonly ... "VACUUM INTO ..."` into
   `$env:TEMP\argus-preview\dashboard\dashboard.db`. Copy `process-rules.json`. Copy the config
   with `port` set to 47617 and `notify.os` set to false.
4. Start it hidden with the three variables set to the copy:

   ```powershell
   $p = "$env:TEMP\argus-preview"
   $env:COORD_HOME = $p; $env:DASHBOARD_HOME = "$p\dashboard"; $env:DASHBOARD_PORT = '47617'
   Start-Process "$p\bin\argus.exe" -WindowStyle Hidden `
     -ArgumentList 'serve','--preview','--port','47617','--data-dir',"$p\dashboard" `
     -RedirectStandardOutput "$p\serve.out" -RedirectStandardError "$p\serve.err"
   ```

   Record the pid from `$p\dashboard\server.pid`.
5. Seed a few typed items on the copy so every layout shows, with the preview binary's own
   `argus coord ...` and the three variables set. Check the three before every write: a missing
   one writes the real board.
6. Open `http://127.0.0.1:47617/` in the lead's browser pane only (`preview_start` with that
   url, or a `.claude/launch.json` entry that attaches to it). Tell the owner it is there and
   end the turn.
7. Stop it with `"$p\bin\argus.exe" stop --data-dir "$p\dashboard"`, by its pid file. Delete the
   preview folder afterwards with the junction-safe rule: it holds no links, check first.

## The demo board: `coord demo seed`

A preview home with real boards copied in shows only what the owner already has. To see every
record kind at once, seed a demo board into a separate preview home:

```text
COORD_HOME=<preview home> coord demo seed [--project <key>]
```

The default key is `demo`. The verb fills that board with:

- an epic with stories and tasks in every stage (each has an epic parent), a bug and a debt;
- test cases tagged AI and manual, plans, and runs in the stages draft, ready, running, reported
  and passed, with a build (launcher) linked to the running and the ready run;
- agents that run, are done, are stopped or failed, with their model and reason, and worktrees
  linked to some of them;
- asks with options, a decision, notices, notes, a to-do, a lock, a log line, an inbox message;
- pull requests that are open, merged and closed, and a process definition (`coord run define`);
- requests to the tessera, brock and archipelia boards (open and done) and one from brock to
  the demo board, so brand marks show on the cards;
- releases under `<preview home>/releases/<key>/`.

Rules, kept by the verb:

- It refuses when `COORD_HOME` is unset, or is the live home (`~/.coord`) or inside it. It never
  writes to the live boards, and the requests between boards live only in the same preview home.
- It refuses when the board already exists. To seed again, delete the folder
  `<preview home>/<key>` first. The request ledger lives on the `portfolio` board of that home.
- The target boards (tessera, brock, archipelia) are created with a lead when the home has none;
  a board that is already there is left as it is.
- Every record is written through the real verbs and board ops, so it passes the same checks as
  a hand made one. A step that fails is printed and the rest still run; the exit code is 1.
- A live process run (the run history row in the server's database) is not seeded: the verb
  has no database. The process definition is.

The worktree folders are made under `<preview home>/<key>-worktrees`.
