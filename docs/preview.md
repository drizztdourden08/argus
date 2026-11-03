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
