<!-- @layer docs @kind doc -->
# Desktop accounts

The owner uses two Max accounts of the chat service on this PC. The Accounts tab of Argus (item I-135) watches
the usage of every account, keeps the sidebar of the Desktop app the same on every account,
and switches Desktop from one account to the other without a new sign-in. It can do that by
itself at a chosen percentage, after every working lead has written a handover, and it starts
each of those leads again on the other account.

The code is the crate `crates/argus-accounts` (the pure parts and the ports) and, in
`crates/argus-server`, `services/accounts*.rs`, `adapters/accounts*.rs` and
`routes/accounts*.rs`. The web side is the Accounts tab of the Overview. The service entry of
`docs/architecture.md` lists the timers, the docs and the routes. The wake side of the handover is
in `docs/wake-policy.md`.

## The data map

All paths are on this PC. `<app>` is the data folder of the Desktop app, found under
`%LOCALAPPDATA%\Claude-Data\claude_*` (or `%APPDATA%\Claude`), or set with `appData`. A scope is
`<account uuid>/<org uuid>`.

| What | Where | Notes |
|---|---|---|
| Chats | `<app>\claude-code-sessions\<account>\<org>\local_*.json` | One file per chat. A chat that has no `cliSessionId` never ran and is skipped. |
| Archive index | same folder, `archived-sessions.idx` | `{v, archived: [ids]}`. |
| Deleted chats | same folder, `deleted_*` | Tombstones. A deleted chat is never brought back. |
| Transcripts | `%USERPROFILE%\.claude\projects\<folder>\<cliSessionId>.jsonl` | Shared on disk by every account, so a copied chat keeps its whole context. |
| Folders and their members | `<app>\Local Storage\leveldb` | Keys `LSS-persisted.dframe-code-sections`, `LSS-persisted.dframe-group-scopes` and `dframe-store`. Locked while the app runs. |
| Scheduled tasks | `<app>\claude-code-sessions\<account>\<org>\scheduled-tasks.json` and `%USERPROFILE%\.claude\scheduled-tasks\<id>\SKILL.md` | A task per scope. The resume task lives here. |
| Desktop login | `<app>\config.json` (keys `oauth:tokenCache`, `oauth:tokenCacheV2`, `lastKnownAccountUuid`) and `<app>\Network\Cookies` | Opaque. Argus copies them and never decodes, shows or logs them. |
| Saved Desktop logins | `<DASHBOARD_HOME>\accounts\logins\<key>\` | One folder per account, user-only access. |
| CLI login | `%USERPROFILE%\.claude-accounts\<key>\.credentials.json` | One per account. Used to read the usage and to find out which organization the account is. |
| Backups of a sync | `<DASHBOARD_HOME>\accounts\backups\<time>\` | A copy of every file a run overwrites, the LevelDB folder and `manifest.json`. |
| Handover files | `~/.coord/<board>/handover.md` | Written by the lead (the Overlord is `portfolio`). |
| Desktop usage log | `<app>\plan-usage-history.json` | The fallback when the usage call fails. |

The folders of the sidebar are not in the chat files. They are in the Local Storage of the app,
with different folder ids on each account, and only a closed Desktop app lets Argus write them.

## Setup

1. Sign in the CLI once per account, each with its own config folder:

   ```text
   CLAUDE_CONFIG_DIR=%USERPROFILE%\.claude-accounts\<key> claude auth login
   ```

   In PowerShell set `$env:CLAUDE_CONFIG_DIR` first. `<key>` is a short name such as `hotmail`.
   Argus matches each folder to a Desktop scope by the organization id that
   `claude auth status --json` prints, and writes the accounts it found into `config.json`
   under `accounts` when that list is empty.
2. Open the Accounts tab. Each account shows its limits, its CLI login and whether its Desktop
   login is saved.
3. The Desktop login of an account is saved by Argus while it is the account Desktop is on. Argus
   saves it in the `save_login` step of every switch away from that account, and on
   `POST /api/accounts/login/save`. So the first switch to an account whose Desktop login was
   never saved needs one manual sign-in in Desktop: the switch syncs the sidebars, finds no saved
   login for the target at `restore_login`, puts the old login back and starts the app as before.
   Sign in to the target account in Desktop once by hand, save its login (the route above, or
   switch away from it), and from then on a switch needs no sign-in.

The login tokens are read only to call the usage endpoint and to renew a login that is about to
end (the CLI renews it, else Argus calls the token endpoint and writes only the access token, the
refresh token and the expiry back). No token is ever logged, put in a doc or sent anywhere else.

## Usage

The usage poll calls `GET https://api.anthropic.com/api/oauth/usage` for each account with the
CLI login. It reads `limits[]` first and keeps limit kinds it does not know. A failed call keeps
the last good read and shows its age. The Desktop usage log is the fallback.

- Every `pollMinutes` (5) per account, and every `fastPollSeconds` (60) once any limit is at
  `fastAt` (80) percent or more.
- Each account is a doc in `Accounts` with its limits, its binding limit, the burn rate in points
  a minute over the last 20 minutes, its CLI login, its Desktop login and its rules.
- Toasts, once per limit window: a warning at `warnAt` (80); for the account Desktop is on and
  with no auto-switch, a proposal to switch at `proposeAt` (95) or at severity critical, naming
  the account with the most room; and "has reset" when a spent limit falls under 50.
- The tray shows one line per account with its weekly percent and "Switch Desktop to ..." entries.

## The sync rules

The sync keeps every account's sidebar the same: the same chats, folders, sub-threads and archived
state. A run is Scan, Compare, Plan, Back up, Apply, Verify.

- Chats. A chat missing on an account is copied with its portable fields. History advances from
  the newest copy. Each account keeps its own permission and tool settings. Tombstones are
  respected. A chat whose copies disagree, or whose transcript is missing or ambiguous, is left
  alone and counted as a conflict or a missing transcript.
- Sub-threads. `spawnedFrom`, `lineageDetached`, `lineageDetachedBy`, `dispatchParentId` and
  `dispatchParentOrigin` are portable. A link on any account wins unless the other side detached it
  on purpose.
- Archived. Merged on its own, the most recent change wins. The setting `archiveRule` is `newest`
  (default) or `anywhere` (archived on one account means archived on all).
- Folders. Matched by name, since the ids differ. A missing folder is created with a fresh id. The
  members and the order follow the newest account. An archived chat keeps its folder.
- Idempotent. Verify plans again after the writes and fails loudly if anything still differs. A
  second run plans no write.

An apply refuses while the app holds the lock of its Local Storage, and refuses a chat file that
changed after the plan was made. While the app runs, Argus only scans: the `drift` doc counts the
differences by kind every 10 minutes and after a poll. A dry run (`POST /api/accounts/sync` with
`dryRun` true, the default) gives the plan only. Each step is a doc in `SyncSteps` and the `run`
doc shows the phase, so the tab follows the run live.

### Backups and undo

Before it writes, a run copies every file it will overwrite and the whole LevelDB folder (without
its `LOCK`) to `<DASHBOARD_HOME>\accounts\backups\<time>\`, with a `manifest.json` that holds the
SHA-256 of each copy. `POST /api/accounts/sync/undo` puts the last backup back (or the one named
by `backupDir`, which must be under the backups folder) byte for byte, after it checks every copy
against the manifest. A file the run created is moved into the backup folder, never deleted.
Nothing is hard deleted and old backups are never pruned by Argus.

## Switching Desktop

The Desktop app runs one account at a time, so a switch is a short relaunch. It starts from the
Switch button of an account card, the tray, a toast link or an auto-switch rule. One switch runs
at a time. Argus refuses a target that cannot receive, that has no known Desktop account, or that
Desktop already runs as.

The steps, in order, each shown in the `switch` doc with its phase:

1. `handover`: ask every working lead for a handover and wait for them (only when the switch was
   started with handover). After 10 minutes, or at once on "Switch anyway", the leads that are
   not ready are cut off.
2. `close`: close the app politely (the main window of each process in its install folder, up to 20
   seconds). Argus never kills a process by name. A chat that looked mid-turn is noted.
3. `save_login`: save the login of the account being left.
4. `sync`: the sync above, so the target opens on the same sidebar.
5. `restore_login`: put the target's saved login in `config.json` (the three keys only) and the
   cookie files.
6. `resume_task`: write the resume task (see below). A failure here is reported and does not undo
   the switch.
7. `start`: start the app through its app id.
8. `confirm`: wait up to 60 seconds for `lastKnownAccountUuid` to name the target.

Cancel works until the app is about to close. A failure after the app closed rolls back: a running
app is closed, the saved login of the old account is put back when the login may have changed
(from `restore_login` on), and the app is started again. The error says whether going back worked.

## The rules

Every account has two rules of its own. They apply while Desktop is on that account.

| Rule | Setting | What it does |
|---|---|---|
| Auto-switch | on or off, and `switchAt` (default 98) | When the binding limit reaches `switchAt` (or is critical, or is 100), a 60 second countdown starts, then the switch runs. The target is the account with the most room: able to receive, under its own `switchAt`, lowest use, then soonest reset. With no such account Argus only tells the owner, once per limit window. |
| Handover | `off`, a percent, or `auto` | Asks every working lead for a handover before the switch. It only runs while auto-switch is on. |

The grace. A banner in every tab counts down, the tray gains "Cancel switch" while the `switch`
doc phase is `countdown`, and the toast opens the tab. At the end Argus checks again: Desktop is
still on the same account, auto-switch is still on, the use is still at the switch point (or the
handovers are all in), and the target still has room. If any check fails it stands down and says
why. Cancel holds the rule off for that account until the use is 5 points under the switch point,
the limit resets, or the rule is turned off and on again. "Switch now" has no countdown.

The handover cycle, per account:

- `off`: the countdown starts at the switch point, with no handover.
- A percent: at that percent every working lead is asked for its handover at once.
- `auto`: once the use is within 10 points of the switch point, or the burn reaches it within 45
  minutes, each working lead gets a `handover-estimate` wake. At the ask point each gets a
  `handover-request` wake.
- When every asked lead is ready, the countdown starts at once, even below the switch point. If
  the switch point comes first, the leads that are not ready are cut off and the countdown
  starts.

A lead is working when its thread is inside a turn or it has a live agent record. An idle lead is
not asked and costs nothing. The lead is told what to do by `coord handover estimate`, `write` and
`status` (the verbs are in `crates/argus-coord/src/verbs/remote/handover.rs`, and only a board's
coordinator may call them).

### The ask point

```text
need   = k x (sum of the leads' estimates)   points the handovers will use
       + burn x T                            what normal work burns meanwhile
       + safety                              0.5 points
ask at = switch point - need                 kept within 30 points below the switch point
burn   = points a minute over the last 20 minutes
T      = longest estimate in minutes + 2 minutes to write the files
k      = calibration, starts at 1.0
```

Worked example. The switch point is 98. Three working leads estimate 0.6, 0.4 and 0.3 points, the
longest takes 8 minutes, the burn is 0.05 points a minute and k is 1.0. The sum is 1.3. T is
8 + 2 = 10 minutes, so the burn adds 0.5. The safety margin adds 0.5. So need is 1.3 + 0.5 + 0.5 =
2.3, and Argus asks at 98 - 2.3 = 95.7 percent.

A lead's own estimate counts while it is under 20 minutes old. Otherwise Argus uses its own
default: the lead's handover `lead` (0.4) plus each live agent by model (`opus` 0.3, `sonnet` 0.1,
`haiku` 0.03, times `longContextFactor` 1.5 over 100k tokens of context). A lead is asked again
only when its agent count changes or its estimate is old, and never more often than every 20
minutes. After each cycle that ended with every lead ready, k moves a quarter of the way toward
actual points used divided by estimated, within 0.5 to 3. The tab shows what the last cycle used.
On a weekly limit one point is a lot of work, so "close to 100" means within about a point.

## The resume task

The chat of a lead carries over because the sync copies its record and the transcript is shared.
What stops is its turn and its agents. Nothing can start a session without a person, so before
the app closes Argus writes a one-off Desktop scheduled task, `argus-resume-<SW-n>`, into the
target scope. It fires 3 minutes after the switch and runs in the Overlord's folder.

The task checks that Argus answers and that the account marked active is not the one being left.
It then sends each lead whose handover is ready or cut off the resume message with `send_message`:
read the handover file, run `coord inbox`, check the agents by real id, restart cut-off agents from
their branches, reopen the board tab, carry on. After 3 minutes it checks the wake listeners and
messages again any lead that did not start. It reports each lead to
`POST /api/accounts/handover` as `resumed` or `not_resumed` and toasts the owner once.

A watcher removes the task after it ran and the leads reported, or 30 minutes after it was due,
and marks the leads that never reported as not resumed. If the task does not run, the leads still
have their chats and branches, and the owner can send the resume text by hand.

## Accounts in a preview

The accounts feature is read-only in a preview and in shadow mode. A sync that applies, an undo, a Desktop switch (also "anyway"), saving a login and writing the accounts settings (shadow only; a preview home is a copy, so its settings may change) answer 403 `read_only`. Dry runs, compare, the GET routes, the handover route and a switch cancel stay open. The switching rules start no countdown, handover cycle or switch, nothing is written to the desktop app's scheduled tasks, the start-up repair of a cut-short switch does not run, and a login is never renewed. Usage is read from the Desktop log only. To see live usage in a preview, start it with `ARGUS_PREVIEW_ACCOUNTS_USAGE=1`; it only reads the usage endpoint with the logins as they are, and a shadow run ignores the variable.

## Routes

| Method | Path | Does |
|---|---|---|
| GET | `/api/accounts` | The accounts and the settings without them. |
| POST | `/api/accounts/usage/refresh` | Poll now. |
| GET | `/api/accounts/compare` | The ordered rows of folders and chats of every account, with a status per cell. |
| POST | `/api/accounts/sync` | `{dryRun}` (default true). Plan or apply. 409 while a run is active. |
| POST | `/api/accounts/sync/undo` | `{backupDir?}`. Put a backup back. |
| POST | `/api/accounts/login/save` | `{key?}`. Save the Desktop login now in use for its account. |
| POST | `/api/accounts/switch` | `{to, handover}`. 202 `{id}`, then follow the `switch` doc. 409 while one runs. |
| POST | `/api/accounts/switch/anyway` | Stop waiting for the handovers. |
| POST | `/api/accounts/switch/cancel` | Cancel a countdown or a switch that has not closed the app. |
| GET | `/api/accounts/cycle` | The current or last handover cycle with its handovers. |
| POST | `/api/accounts/handover` | `{action, board, thread, ...}`: `estimate`, `write`, `status` from `coord handover`; `resumed`, `not_resumed` from the resume task. |
| GET, POST | `/api/accounts/settings` | The settings; a POST replaces the keys sent. |
| GET | `/api/accounts/tray` | What the tray menu needs. |

The live data is in `_portfolio` docs: the colls `Accounts` (one doc per account key),
`AccountsState` (`drift`, `run`, `switch`, `cycle`), `SyncSteps` and `Handovers` (one `H-n` per
lead).

## Config

The key `accounts` of `config.json`. Every field has a default.

| Key | Default | Meaning |
|---|---|---|
| `enabled` | true | Run the service. |
| `appData` | null | The Desktop data folder; null finds it. |
| `cliRoot` | `%USERPROFILE%\.claude-accounts` | Where the CLI config folders are. |
| `pollMinutes` | 5 | Minutes between usage reads. |
| `fastPollSeconds` | 60 | Seconds between reads once a limit is at `fastAt`. |
| `fastAt` | 80 | Percent from which the fast poll runs. |
| `warnAt` | 80 | Percent from which the owner is warned. |
| `proposeAt` | 95 | Percent from which a switch is proposed. |
| `archiveRule` | `newest` | `newest` or `anywhere`. |
| `estimateDefaults` | see above | `opus`, `sonnet`, `haiku`, `lead`, `longContextFactor`, `safety`. |
| `k` | 1.0 | Calibration of the estimates, learned. |
| `accounts` | discovered | Per account: `key`, `label`, `color`, `desktopAccount`, `desktopOrg`, `cliDir`, `canReceive`, `rules` (`autoSwitch`, `switchAt`, `handover`). |

## Not done yet

These are the Later list of the plan:

- A one-click switch from the toast itself. A toast button can only open a URL today, so it opens
  the tab and the owner confirms. A click-back needs an activation handler in `argus-native`.
- Cowork sessions (`local-agent-mode-sessions`) and pinned chats are not synced.
- More than two accounts. The design takes a list; only the compare view needs a column picker.
- Running a CLI task as a chosen account from Argus, using its CLI folder.

Two limits stay by design. The usage endpoint is what the CLI uses, not a published API, so Argus
reads it loosely. The Desktop storage belongs to the app and can change with an update; Argus
refuses to write a shape it does not know and says so.
