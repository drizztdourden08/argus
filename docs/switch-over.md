<!-- @layer docs @kind doc -->
# Switch-over runbook

This is the order of work that moves the machine from the Node dashboard to Argus. It follows
the switch-over section of `docs/architecture.md`, the checklist of `docs/parity.md` and the shim
phases of `docs/coord-port.md`. Every step names who does it, the exact commands, how to verify
and how to go back. Nothing here is done yet. The Node dashboard stays the live one until step 4.

## Who does what

- The Argus lead and its agents run the shadow, the comparisons, the coord rollout and the cut.
  The cut itself is one lead turn: the commands in step 4, then the checks of step 5.
- The Overlord owns the config repo (`X:\ai-config`): the `dashboard` shim, the rendered coord
  files, the PATH shims and the later retirement of the Node code. The lead asks it with
  `coord lead portfolio "..."`.
- The owner tests and decides. Each point that needs a go is marked **Go** and collected in the
  last section.

Every sub-agent follows the project instructions file: no test run by a builder, one haiku agent for the checks,
no process stopped by name, no window put on the owner's desktop. Hidden background processes
are fine. Every stop below goes through a stop verb that works from a pid file.

## Names used below

```powershell
$argus  = "$env:LOCALAPPDATA\Programs\argus\argus.exe"   # the installed copy, see "Install the binary"
$node   = "$HOME\.coord\dashboard"                # the live Node home, the future Argus home
$shadow = "$HOME\.coord\argus-shadow"             # the shadow home
$cfg    = 'X:\ai-config'                          # the config repo
```

## Blockers found in the code

These are gaps between this plan and the code at commit `c45f9da`. Items 1 to 3 must be closed
before the cut. Items 1 and 3 are fixed.

1. The port. Fixed in 813fab3. `argus serve`, `argus start`, the tray and the MCP bridge now use the
   `port` of the home's `config.json` (47615 by default), with `--port` and `DASHBOARD_PORT`
   overriding as in Node. The `NODE_PORT` special case is gone. Safety before the cut stays with
   the node guard: `serve` refuses a home a live Node server uses, and every client verb
   refuses a Node server. A new shadow gets a copied config whose port is 47616 (its one
   change), so `serve --shadow` needs no `--port`; the commands below still pass it, which is
   harmless.
2. The shim. Neither side has one. `X:\ai-config\dashboard\bin\dashboard.mjs` has no forward
   block and no engine key, and Argus has nothing to write. Step 4 needs the Overlord to land one
   (the design is in step 4, on the model of `shared/tools/coord/forward.mjs`).
3. The backup. Fixed in 813fab3. The first `argus serve` of a home with no `argus.marker` file (and
   not a shadow) takes `dashboard.db.node-<UTC stamp>` with `VACUUM INTO` before anything
   writes, then writes the marker. Step 4 still copies by hand, which does no harm.
4. The `exe` key. Argus writes `exe` into the config it loads, but `forward.mjs` reads
   `exe` from the Node home before Argus has ever run there. Step 3 adds it by hand. Node keeps
   unknown keys, so it survives Node's own rewrites.
5. `coordVerbs` matches the word typed, not the group. `ask`, `decide`, `notify`, `message`,
   `answer`, `todo`, `restore`, `trash`, `dispatch` and `wake-fault` are separate words. A list
   of groups misses them. Step 3 lists them by hand and ends on `["*"]`.
6. The shadow refuses writes (`read_only`): owner posts, test results, runs, Play, worktree
   removal and the MCP write tools. The trial of step 2 cannot try them. The sandbox of step 1
   covers them on a copy of the data before the cut.

No `dashboard` verb is missing in `argus`: serve, start, stop, status, open, logs, resync, token,
tray, mcp, wake-stream, install-mcp and autostart all exist. Two small output differences are
worth a look in step 5. `argus --version` is the clap line, where Node prints the bare version,
and `argus help` is the clap help text.

## Install the binary

Do this once before step 1, and again at the cut with the cut build.

A running `argus.exe` cannot be overwritten on Windows, and the config key `exe` should name a
path that never changes. So the live copy is not `target\release`, which every build overwrites.
Build, then run `self-install` from the fresh build.

```powershell
pnpm build
cargo build --release
X:\argus\target\release\argus.exe self-install
& $argus --version
```

`argus self-install [--dir <folder>] [--print]` copies the running binary to
`%LOCALAPPDATA%\Programs\argus\argus.exe` (or the `--dir` folder). The web build is inside the
binary, so that one file is the whole install. `--print` shows the path and copies nothing.
The verb prints the path and the exact config line to add, `"exe": "<path>"`, and edits no
config itself.

The in-use rule: the copy goes to `argus.exe.tmp` beside the target and is renamed over it. A
running `argus.exe` cannot be replaced on Windows, so then the rename fails, the temp file is
removed, the installed copy is left as it was, and the verb exits 1 with a sentence that says to
run `argus stop` and try again. It never stops anything itself. A target that already holds the
same bytes is left alone ("already up to date"), and so is a run from the installed copy.

To update later: `argus stop` (or `dashboard stop`), `self-install` from the new build, start.
The web build is embedded at build time, so `pnpm build` always comes first.

## 1. Clean shadow comparison

Who: the Argus lead's agents. A haiku agent runs the commands and reports the diff.

Start a fresh shadow. The shadow home is Argus's own folder, so it may be deleted and re-seeded.
The Node home is only read. Never delete or edit it here.

```powershell
$env:DASHBOARD_HOME = $shadow
& $argus stop                       # prints "dashboard is not running" when none runs
Remove-Item -Recurse -Force $shadow -ErrorAction SilentlyContinue
Start-Process $argus -WindowStyle Hidden -ArgumentList 'serve','--shadow','--port','47616',
  '--data-dir',$shadow
Remove-Item Env:DASHBOARD_HOME
```

The first start copies the Node database (`VACUUM INTO`), `config.json` and `process-rules.json`
into the shadow home. It reads the real boards in `~/.coord` and writes none of them.

Verify the shadow is up:

```powershell
Invoke-RestMethod http://127.0.0.1:47616/api/health | Select-Object ok,impl,shadow,schema
```

Expect `impl` argus, `shadow` True and `schema` 6 (the Node value).

Compare. The Node server is the left side.

```powershell
& $argus parity --left http://127.0.0.1:47615 --right http://127.0.0.1:47616 `
  --out "$shadow\parity.txt"
```

`--json` gives the machine form. Exit 0 means every route is equal. Exit 1 means a difference or
a route missing on one side.

The gate:

- The doc collections of every board and `_portfolio` (`threads items tests entries locks
  actions agents log meta cards builds launchers registry worktrees`), `/api/projects`,
  `/api/p/<key>/launches`, `/api/procs/rules`, `/api/notify/settings`, the MCP `initialize` and
  the MCP `tools/list` are equal. Zero real differences.
- Live noise is allowed, and only that. The shadow keeps one copy of the Node database from its
  start and takes no outward action, so these routes can differ in rows that Node made after the
  seed: `/api/runs` and `/api/p/<key>/runs` (the shadow starts none), `/api/p/<key>/wakes`,
  `/api/p/<key>/wakes/listeners`, `/api/presence`, `/api/wake-faults`, `/api/notify` and the
  `notifications` doc, `/api/activity` and the `activity` doc, `/api/procs` and the `procs`
  doc, `/api/agents/reconcile`, and the `agents` and `meta` docs where the parity rules do not
  already ignore the field. The parity rules already set aside seq numbers, timestamps within
  2 s, the whole `live` object of agents, the listener-derived fields of the `registry` doc
  (`listening`, `listenerLabel`, `state`, `fault`, `since`, `deaf`, `boardOpen`, `boardSince`,
  `boardRequired`) and the lifecycle lines of the activity feed.
- The second shadow run found three more sources of noise. The `registry` doc and its stream
  snapshot follow who listens to each server, which the rules above now set aside. The wakes
  list differs when one side has pruned past its 50-row cap and the other has not, so the oldest
  rows are absent on one side. The worktree scan can differ by timing when a scan lands between
  the two reads. `--skip <word>` leaves out the matching routes and also the matching stream
  snapshot frames (`--skip wakes` drops `snapshot wakes`), so a known noise source is dropped
  from both parts of the report.
- A difference on a noise route counts only when it is a row or field that exists on both sides
  and disagrees, and only when the row is older than the shadow's start. A row that is newer
  than the start, or absent on one side because of the seed, is noise. The lead writes the
  verdict on the board item, with the report path.
- The comparison is run twice: right after a fresh seed, and again after a full working day with
  the shadow left running (the checklist asks for a day). The second run is the real gate.

Run the Rust and web suites through the haiku agent as the checklist says (`cargo fmt --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` with
`ARGUS_COORD_CLI` set, `pnpm lint`, `pnpm lint:md`, `pnpm structure`, `pnpm test`).

The sandbox, which covers what the shadow cannot. The shadow refuses writes, so the write paths
and the Node-reads-Argus-database rollback are tried on a throwaway copy of the data on port
47617. It is isolated: its own `COORD_HOME`, a copy of the boards, a copy of the database, OS
toasts off, no session listening on it. Its run defs and worktree rows still point at real
folders, so nothing in it presses Play on a real launcher or removes a real worktree. The whole
sequence below has been rehearsed end to end: Argus not in shadow mode on 47617 with writes,
Node on 47618 reading the database Argus wrote, then the shim path.

Build the sandbox. The sandbox runs a COPY of the binary, never `target\release` while a
rebuild may follow, and never the installed copy. `argus self-install --dir <sandbox>` makes
that copy. The boards are copied with bash `cp -r`, leaving out `dashboard`, every
`argus-shadow*` folder and any folder whose `board.json` is damaged. The database is a
read-only `VACUUM INTO` of the Node database, so the Node home is only read.

```bash
sb="$TEMP/argus-rehearsal"; mkdir -p "$sb/dashboard" "$sb/bin"
cd "$HOME/.coord"
for d in */; do d=${d%/}
  case "$d" in dashboard|argus-shadow*) continue;; esac
  # skip a folder whose board.json is damaged (does not parse as JSON)
  [ -f "$d/board.json" ] && ! node -e 'JSON.parse(require("fs").readFileSync(process.argv[1]))' "$d/board.json" 2>/dev/null && continue
  cp -r "$d" "$sb/"
done
sqlite3 -readonly "$HOME/.coord/dashboard/dashboard.db" "VACUUM INTO '$sb/dashboard/dashboard.db'"
cp "$HOME/.coord/dashboard/process-rules.json" "$sb/dashboard/"
```

```powershell
$sb = "$env:TEMP\argus-rehearsal"
$c = Get-Content "$node\config.json" -Raw | ConvertFrom-Json
$c.port = 47617; $c.notify.os = $false
$c.PSObject.Properties.Remove('dashboardEngine')      # no engine key: the sandbox is Argus only
$c | ConvertTo-Json -Depth 10 | Set-Content "$sb\dashboard\config.json"
& $argus self-install --dir "$sb\bin"                 # the copy the sandbox runs
$sbx = "$sb\bin\argus.exe"
$env:COORD_HOME = $sb; $env:DASHBOARD_HOME = "$sb\dashboard"; $env:DASHBOARD_PORT = '47617'
Start-Process $sbx -WindowStyle Hidden -ArgumentList 'serve','--port','47617' `
  -RedirectStandardOutput "$sb\serve.out" -RedirectStandardError "$sb\serve.err"
```

Then drive it, with the three env variables still set. Each line has an expected result:

```powershell
coord thread register sandbox-cut --subject rehearsal --project argus
coord item add "rehearsal item" --project argus --as sandbox-cut
coord run define rehearse-echo --cmd "cmd /c echo hi" --cwd $sb --kind task
coord run start rehearse-echo --wait                  # R-1, exit 0
coord ping "x" --level ping                           # "recorded, no toast"
```

- An owner post: `POST /api/p/argus/entries` with a kind `edit` body: `id`, `createdAt`,
  `target` (`{c: "items", id}`), `set` (`{title}`), `baseRev` and `prev`, as the web test builds
  it. The item shows the new title.
- MCP: `tools/list` through `POST /mcp` with `Accept: application/json` lists 23 tools.

Stop it with `& $sbx stop`. Rollback rehearsal: start the Node server on the database Argus just
wrote, on port 47618 and with the engine forced to Node. Node `start` and `stop` never touch
the tray.

```powershell
$env:DASHBOARD_PORT = '47618'; $env:DASHBOARD_ENGINE = 'node'
$d = "node $cfg\dashboard\bin\dashboard.mjs"
Invoke-Expression "$d start"
Invoke-Expression "$d status"                         # running on 47618
Invoke-RestMethod http://127.0.0.1:47618/api/health   # schema 6
# the item: /api/p/argus/db/items/<id>, and R-1 listed in /api/p/argus/runs
Invoke-Expression "$d stop"
```

Sandbox cut, the shim path in small. Node's `dashboard.mjs` starts the Argus binary when the
engine says so. Here `ARGUS_EXE` names the sandbox copy:

```powershell
$env:DASHBOARD_PORT = '47617'; $env:DASHBOARD_ENGINE = 'argus'; $env:ARGUS_EXE = $sbx
Invoke-Expression "$d start"
Invoke-Expression "$d status"                         # impl argus
Invoke-Expression "$d resync --hard"
foreach ($h in 'argus','argus-app','overlord','rotp') {
  curl.exe -s -o NUL -w "$h %{http_code} %{redirect_url}`n" `
    --resolve "$h.localhost:47617:127.0.0.1" "http://$h.localhost:47617/"
}
Invoke-Expression "$d stop"
```

Delete `$sb` and clear `COORD_HOME`, `DASHBOARD_HOME`, `DASHBOARD_PORT`, `DASHBOARD_ENGINE` and
`ARGUS_EXE` afterwards. Rollback of this step: stop the sandbox servers (`$sbx stop`,
`$d stop`, each with the env above) and stop the shadow (`$env:DASHBOARD_HOME = $shadow; & $argus
stop`). Nothing outside `$shadow` and `$sb` was written.

## 2. Owner trial of the shadow build

Who: the owner. The lead hands over the command and ends its turn. It never opens a window.

The owner runs this in his own terminal. It stays in the foreground, and Ctrl+C ends it:

```powershell
& "$env:LOCALAPPDATA\Programs\argus\argus.exe" serve --shadow --port 47616 --data-dir "$HOME\.coord\argus-shadow"
```

Then he opens `http://127.0.0.1:47616/` beside the Node page on 47615. What to look at:

- The overview tabs: Dashboards, Processes, Activity, Notifications. Same cards, counts and order
  as Node.
- Each board page and each tab: Activity, Kanban, Tests, Builds, Worktrees, Agents. The board
  wears its own palette and the page follows when he switches dashboard. The overview wears the
  basic one.
- A board on its alias host: `http://rotp.localhost:47616/` and `http://argus.localhost:47616/`
  (the overview) and `http://argus-app.localhost:47616/` (the Argus board).
- Markdown, proof images and the log terminal on a board with real runs.
- Writes will answer "shadow mode" and change nothing. That is expected.

He answers with one verdict. A defect goes to the board as an item and the lead fixes it, then
repeats step 1. **Go:** the owner's "looks right" is the go to start step 3.

Rollback: Ctrl+C. Node was never touched.

## 3. The coord engine rollout

Who: the Argus lead, one group at a time, each after its differential run. The Overlord must have
landed and rendered `forward.mjs` in every project first (phase A). Check one checkout per
project for `.claude\tools\coord\forward.mjs` and a `forward(import.meta.filename)` line in
`coord.mjs`. A project not rendered yet runs Node whatever the config says.

Preparation. Install the stable copy (`argus self-install` from the build, see "Install the
binary"), which prints the line to add. Add the `exe` key by hand, so the forward block finds the
binary before Argus has ever run in the Node home. Edit `$node\config.json` and add the printed
line, `"exe": "C:\\Users\\drizz\\AppData\\Local\\Programs\\argus\\argus.exe"`, and put `coordEngine` in the same edit once a step is ready. The server need not stop. Write the
file in one save. Node re-reads it per call and keeps the keys it does not know.

The gate of each step is the same:

- The Rust ports for those verbs pass, and the Node suites for them pass with
  `COORD_ENGINE=argus ARGUS_EXE=$argus node --test <file>` from `$cfg\dashboard`.
- `crates/argus/tests/coord_diff.rs` and its corpus lines show no difference for the verbs.
- One day of the real-usage replay on fresh copies (layer 4 of `docs/coord-port.md`), kept
  outside the repo.
- Before the first step, the port fix of blocker 1 is landed and installed. It is what makes
  every launcher start Argus on the port of the config.

The order and the lists. Each step replaces the value of `coordVerbs` in `$node\config.json`
and keeps `"coordEngine": "argus"`.

```text
3a read verbs   activity, proc, wake-fault, check, tick, board, dashboard
                (the server clients that only read, plus the read-only board verbs)
3b write verbs  run, ping, lead, portfolio, worktree, then item, test, entry, ask, decide,
                notify, message, answer, action, todo, lock, log, inbox, agent, build,
                launcher, thread, dispatch, purge, restore, trash, sync
3c hooks        hook
done            ["*"]
```

Write each step as the full array, for example for 3a:

```json
"coordEngine": "argus",
"coordVerbs": ["activity", "proc", "wake-fault", "check", "tick", "board", "dashboard"]
```

3b is split in three or four smaller edits, in the order of the table in section 1 of
`docs/coord-port.md` (server clients, work docs, agents and bundles, threads and board). Use the
typed word of every alias. Keep `hook` out until 3c. The hooks run on every tool call of every
session, so they go last, after a week of 3b if the lead can afford it, and always after the
latency bench (`crates/argus-coord/tests/latency_bench.rs`) passes its targets.

Verify after each edit, from any project:

```powershell
coord activity --limit 3
Get-Content "$node\logs\coord-shim.log" -Tail 5    # no new "fallback to node" lines
```

The shim log names every spawn failure. An exit 120 is silent by design and means the verb ran in
Node, so a verb that stays slow or Node-shaped is checked with `COORD_ENGINE=argus coord <verb>`.

Flip back instantly, for a verb or for everything. This is a config edit, with no restart and no
render:

```json
"coordEngine": "node"
```

An env override works for one shell: `$env:COORD_ENGINE = 'node'`. Argus and Node write the same
`board.json` bytes under the same lock, so a verb run on one engine is readable by the other.
Nothing needs repair. The first failed differential line or board diff after a step is the
trigger to flip back, then fix, then repeat the step.

**Go:** the Overlord agrees the contract once (the keys `coordEngine`, `coordVerbs`, `exe`, the
exit 120, `ARGUS_COORD_PARENT`, the shim log) and confirms the render reached every project. The
lead then moves through 3a and 3b on its own. 3c (hooks) needs the Overlord's explicit go,
because every session depends on it.

## 4. The cut

Who: the Argus lead, in one turn, with the Overlord's shim already landed and the owner's go in
hand. Pick a quiet moment. The gap is a few seconds. Sessions reconnect on their own.

**Go (owner):** the lead asks one click question, `coord ask`, with "cut now" as the recommended
option, and cuts when he answers. **Go (Overlord):** the shim below must be landed and rendered
with the engine off first. The Overlord is told with `coord lead portfolio` before and after.

Before the window (done days earlier, no impact):

- The cut build is installed at `$argus` (see "Install the binary") and blockers 1 and 2 are
  closed. The steps 1 to 3 gates were green on that same build.
- The Overlord has landed the dashboard shim. Design, on the model of `forward.mjs`: a block at
  the top of `$cfg\dashboard\bin\dashboard.mjs` that reads `dashboardEngine` and `exe` from the
  Node home config. When `dashboardEngine` is `argus` (or `DASHBOARD_ENGINE=argus`) and the exe
  exists, it runs `exe <same arguments>` with inherited streams, `windowsHide`, and exits with
  its code. Otherwise it carries on in Node. `DASHBOARD_ENGINE=node` always runs Node. Every
  `dashboard` caller then goes through it unchanged: the PATH shims in `~\.rotp-cli\bin`, the
  `.mcp.json` entries (`cmd /c dashboard mcp --project <key>`), `coord dashboard ...`, the
  Monitor lines (`dashboard wake-stream ...`) and the `node <installDir>\bin\dashboard.mjs
  start` that the coord client runs. `installDir` is kept, so the file stays where it is. The
  contract is a new config key, so it is an Overlord decision. The default stays `node`.
- `render.mcp` passes in the config repo with the shim in place. The Overlord runs it.
- The final parity run of step 1 is clean (a fresh shadow, then compare, then stop the shadow).

In the window. The lead's haiku agent runs these in one script and stops at the first failure.

```powershell
$env:DASHBOARD_ENGINE = 'node'                      # the Node bin for every command below
$d = "node $cfg\dashboard\bin\dashboard.mjs"
```

1. Check nothing important is running, because stopping the Node server ends its runs:

   ```powershell
   (Invoke-RestMethod http://127.0.0.1:47615/api/health).runs
   ```

   Wait for `running` 0, or let the owner decide. Say in the Overlord note that sessions will
   reconnect.
2. Mark the dashboard as closed on purpose. This keeps the session hooks and the MCP bridges
   from restarting Node while the cut runs. They honour `quit.flag` until the next start:

   ```powershell
   Set-Content "$node\quit.flag" ('{"at":"' + (Get-Date).ToUniversalTime().ToString('o') + '","pid":0}')
   ```

3. Stop the tray and the server with their own verbs, in this order (the tray first, because
   `tray stop` closes only the icon and the server keeps running):

   ```powershell
   Invoke-Expression "$d tray stop"        # tray stopped, or "tray is not running"
   Invoke-Expression "$d stop"             # dashboard stopped
   Invoke-Expression "$d status"           # exit 1, "dashboard is not running"
   ```

   If `stop` reports "the server did not stop", stop here and look. Do not stop it by name.
4. Take the backup. After a clean Node stop the WAL is 0 bytes and `dashboard.db-shm` remains.
   Check, then copy. This is a second copy: the first start of a new home also takes
   `dashboard.db.node-<UTC stamp>` itself (blocker 3, fixed), so the manual copy is a belt and
   braces step:

   ```powershell
   $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
   Get-ChildItem "$node\dashboard.db*" | Select-Object Name,Length
   Copy-Item "$node\dashboard.db"  "$node\dashboard.db.node-$stamp"
   Copy-Item "$node\config.json"   "$node\config.json.node-$stamp"
   ```

   If `dashboard.db-wal` still has bytes, copy it too as `dashboard.db.node-$stamp-wal` and
   tell the lead before going on.
5. Switch the engine, in one save of `$node\config.json`. The server is stopped, so nothing
   rewrites the file. Add `"dashboardEngine": "argus"`. Keep `exe`. Keep `installDir`
   (`X:\ai-config\dashboard`): Argus keeps a given value and the coord client still needs
   `bin\dashboard.mjs` there. Leave `port` at 47615.
6. Start Argus through the shim, which exercises it. The database is opened as it is
   (`PRAGMA user_version` 6, no migration), the config `exe` is rewritten, `installDir` kept:

   ```powershell
   Remove-Item Env:DASHBOARD_ENGINE
   dashboard start                         # dashboard running at http://127.0.0.1:47615/ (pid n)
   dashboard resync --hard                 # the one hard resync of the plan
   dashboard tray start                    # tray started (pid n); clears quit.flag
   dashboard status                        # impl argus, tray running
   ```

   `dashboard start` clears `quit.flag` itself. If the start fails, go to the rollback below.
7. The alias hosts and the board shortcuts. `curl.exe` does not resolve `.localhost` on every
   Windows build, so pin the name:

   ```powershell
   foreach ($h in 'argus','argus-app','overlord','rotp') {
     curl.exe -s -o NUL -w "$h %{http_code} %{redirect_url}`n" `
       --resolve "$h.localhost:47615:127.0.0.1" "http://$h.localhost:47615/"
   }
   ```

   Expect 200 for `argus` (the overview), 302 to the board for `argus-app`, `overlord` and
   every project key.
8. Tell the Overlord: `coord lead portfolio "Argus is live on 47615. Node is stopped. Shim engine
   argus."`

MCP configuration. Rendered projects already call `dashboard mcp`, so they follow the shim with
no change. For a folder the config repo does not render,
`argus install-mcp --code --dir <folder> --project <key>` prints the `.mcp.json` entry. Sessions
that are already open hold a stdio bridge of the Node build, which keeps working because it
talks HTTP to the same port. New sessions get the Argus one.

The desktop app entry is prepared for the owner and is NOT run. Run it from the installed copy,
because the entry names the binary that runs the command:

```powershell
& $argus install-mcp --desktop            # prints the entry, writes nothing
& $argus install-mcp --desktop --write    # the write
```

`--write` merges only the `coord-dashboard` key into
`%APPDATA%\Claude\claude_desktop_config.json` and keeps every other key. Before it writes, it
copies the file as it is to `claude_desktop_config.json.bak.<UTC stamp>` beside it (a missing
file is created with no backup), and it prints the backup path and asks for a restart of the
desktop app. To go back, copy the backup over the file. **Go (owner):** before `--write`, because it
changes a tool setting outside the repo.

Autostart. Argus-side autostart is the tray, started by the MCP bridge and the hooks. A login
launcher exists only if someone installed it. It is not installed on this machine today (the
Startup folder holds no `coord-dashboard.vbs`). If a later check finds one, `argus autostart
status` tells whether it still points at Node, and `argus autostart install` rewrites it for
`argus.exe tray start` under the same name.

Rollback of the cut: see step 5.

## 5. Post-cut checks and the rollback to Node

Who: the Argus lead's haiku agent runs the commands and reports failures only. The owner
confirms what he sees.

Checks, in this order. Stop at the first failure and roll back if it is not a one-line fix.

1. Health.

   ```powershell
   Invoke-RestMethod http://127.0.0.1:47615/api/health | Select-Object ok,impl,schema,version
   dashboard status --json
   dashboard logs --lines 30
   ```

   Expect `impl` argus, `schema` 6, no errors in the log.
2. Every board page answers, and renders. The status check is a loop. The render check is a
   throwaway headless script (Playwright, no window) that opens each page and fails on a
   console error:

   ```powershell
   foreach ($k in (Invoke-RestMethod http://127.0.0.1:47615/api/projects).key) {
     "$k " + (curl.exe -s -o NUL -w '%{http_code}' "http://127.0.0.1:47615/p/$k/")
   }
   ```

3. Wakes reach sessions. The Overlord sends `coord lead argus "cut check"`. The Argus lead's
   Monitor line prints the wake. Then `Invoke-RestMethod http://127.0.0.1:47615/api/p/argus/wakes/listeners`
   lists a listener per live session, and every Monitor `wake-stream` reconnected. Wakes queued
   before the cut replay. If a session's listener is missing, its next tool call runs the hook
   and reconnects it.
4. A hook round trip. Any session prompt runs the prompt and tool hooks. The thread's `lastSeen`
   and activity move on its board (`coord thread list`). After step 3c these hooks run in Rust.
5. The write verbs against Argus: `coord item add --title "cut check"`, then the item shows on
   the board page, then `coord item rm <id>`. And `coord run start <defined run> --wait`.
6. MCP tools, from a session that uses them: `health`, `board_summary`, `list_items`, and one
   write such as `post_entry` on a throwaway thread. The `tools/list` count is the Node count.
7. Toasts and tray. `coord ping "cut check"` shows a toast (the owner confirms; links open the
   alias host on 47615). `dashboard tray status` says running, and the owner sees the icon. The
   icon turns grey with a red badge on stop and back on start.
8. The Node side is quiet: no `node` process holds `dashboard.mjs serve`. The lead checks
   `Get-CimInstance Win32_Process` for command lines containing `bin\dashboard.mjs serve` and
   reports them, and does not stop anything by name.

One day later, `dashboard status` shows the same pid unless the owner stopped it, the log has
no repeated warnings, and the notification record is filling.

Finished agents lose their live counters at the cut, as at any Node restart: a fresh server only
tracks agents that ended within `recentMs`. That is expected, not a regression.

Rollback to Node. The database stays readable by Node because Argus adds no migration and the
board files are never rewritten by Argus on its own. The order matters: the engine flips back
before anything is started, or `dashboard start` would start Argus again.

```powershell
New-Item "$node\quit.flag" -Force | Out-Null           # hooks and bridges leave it alone
dashboard tray stop
dashboard stop                                          # still goes through the shim, so Argus
dashboard status                                        # exit 1
```

Then edit `$node\config.json` once: `"dashboardEngine": "node"` (and `"coordEngine": "node"` when
step 3 had been finished). Then:

```powershell
dashboard start        # the Node bin: dashboard running at http://127.0.0.1:47615/
dashboard tray start   # the PowerShell tray
dashboard status
```

Check that a board's items match, and that `dashboard logs` shows no schema error. If Node
refuses the database, stop it and restore the backup: copy
`$node\dashboard.db.node-<stamp>` over `$node\dashboard.db`, delete the leftover
`dashboard.db-wal` and `dashboard.db-shm` first, then start Node. That loses only the rows the
database alone held since the cut (runs, wakes, notifications, activity). The boards are not
affected. The `exe` and `palette:<key>` entries Argus left are ignored by Node.

## 6. Retiring Node later

Who: the Overlord, with the Argus lead. Nothing here happens at the cut. **Go (owner):** one
question before any deletion. **Go (Overlord):** it owns the config repo.

When: after phase C of the coord shim (`coordVerbs` is `["*"]`), at least two weeks of normal use
at the cut state with no rollback, the owner's confirmation, and a last full test run of both
suites. The backups `dashboard.db.node-*` stay until the owner says they can go.

What stays in the config repo:

- `dashboard\bin\dashboard.mjs`, reduced to the forward stub, and `installDir` in `config.json`,
  until the coord client reads `exe` instead (`dashboard-client.mjs`, an open question in
  `docs/architecture.md` that the Overlord answers).
- `shared\tools\coord\forward.mjs` and the coord files that the forward needs, until phase D.
- `render.mcp` and `render.publish` and the guard tests, which test the config repo itself.
- A git tag on the last commit that holds the whole Node dashboard (for example
  `node-dashboard-final`), so a rollback after the deletion is a checkout.

What is removed, in one commit in the config repo, after the tag:

- `dashboard\src`, `dashboard\pages`, `dashboard\static`, `dashboard\tools` and the tray script.
- `dashboard\test`, which was the specification. The parity matrix now lives on the Argus side.
  Its cases that still have no Argus port (`n/a` rows in `docs/parity.md`) are the only ones
  that stay.
- The Node verb modules of the coord CLI, in phase D, after the hooks and the PATH shims call
  `argus.exe coord` directly (`ai\install-cli.mjs`, whose `coord.cmd` starts Node twice today).

After that, the `dashboard` PATH shims call `argus.exe` directly, and the forward stub can go
too, once nothing starts `node ...\bin\dashboard.mjs`.

## Points that need a go

- Overlord: land the `dashboard` shim and its `dashboardEngine` key in the config repo, engine
  off (step 4, before the window). Also the contract of step 3 and the proof that `forward.mjs`
  reached every project.
- Overlord: the go for 3c, the hooks.
- Overlord: run `render.mcp` and `render.publish` with the shim (the gate of step 4); told before
  and after the cut.
- Owner: the verdict of the shadow trial (step 2), which starts step 3.
- Owner: the cut itself (step 4), asked as one click question with "cut now" as the default.
- Owner: the write into the desktop app config by `install-mcp --desktop --write`.
- Owner and Overlord: retiring the Node code, in step 6.

The lead decides alone, and records it with `coord decide`: the install path of `argus.exe`, the
split of the 3b steps, the timing of the cut inside the owner's go, and the choice to add an
`argus` verb for the backup copy.
