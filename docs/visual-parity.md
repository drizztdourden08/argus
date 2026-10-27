<!-- @layer docs @kind doc -->
# Visual parity: Node dashboard against Argus

Side by side screenshots of the Node dashboard (port 47615) and Argus in shadow mode (port 47616),
taken headless at 1440x900 and 420x900, dark and light, on the boards RotP, Argus and Brock. Each
section lists the differences, the most visible first, with the fix in Argus terms. A fix that needs
a piece Tessera does not have yet says so. The gallery `docs/visual-audit/index.html` shows every
pair with the same numbered list under it.

## Page shell, theme and density (every screen)

1. Light theme. Node follows the system: with a light system theme every Node page is light (white
   surfaces, dark text, the same status tones). Argus stays dark at every viewport. Fix: give
   TesseraProvider a light theme and switch it on `prefers-color-scheme`, for the basic palette and
   for each project palette. This needs a new Tessera piece.
2. Page frame. Node runs edge to edge with a 16px gutter and a plain title row. Argus adds a 24px
   gutter and draws the title inside a 64px ContentHeader panel with a gradient backdrop, so every
   page starts about 60px lower. Fix: `OverviewPage.tsx` and `BoardPage.tsx` use `ContentHeader
   compact` with no `backdrop`, and PageFrame uses the 16px gutter token.
3. Density. Node rows are dense: an activity row is 34px, a notification 52px, an agent 45px, a test
   48px. Argus rows are 60px, 83px, 63px and 110px. On the Notifications tab Node shows 12 rows
   above the fold and Argus 6. Fix: wrap the app in `DensityScope` compact and build rows from
   `ListItemRow` on one line, as listed per screen below.
4. Section and column labels. Node uses small uppercase letter-spaced headers (STATUS, AGENT, PATH)
   and section titles with a middle dot ("Steps · 4", "Tests · 0/0 passing"). Argus uses sentence
   case table headers and commas ("Acceptance criteria, 0/0 met"). Fix: SectionHeader and the
   DataTable header in their uppercase caption style, and the Node wording with the middle dot.
5. Time format. Node uses short relative ages everywhere ("26m", "7h", "3d ago", "48h ago"). Argus
   mixes long forms ("26 minutes ago", "2 days ago", "Yesterday") and durations ("4d 18h", "15h
   21m"). Fix: one formatter (the overview `ShortAge`) for every age in the app, with Node rounding
   (Node says 3d where Argus says 2 days).
6. Body font and size match (Inter, 14px) on both. The difference is weight: Argus sets event and
   notification text in bold where Node uses regular weight.

Screenshots: `visual-audit/w1440-light-overview-dashboards.png`,
`visual-audit/w1440-light-rotp-kanban.png`, `visual-audit/w1440-dark-overview-notifications.png`.

Status (fix-board-shell): 1 is dropped, dark only by owner rule (Tessera lead confirmed). 2 done for
boards: `shell/PageFrame` is a Tessera ScrollArea with a 16px gutter (12px under 720px) and no top
padding, and the board header is its own compact row (`BoardHeader`). The overview header is the
overview thread's. 3 done: `PageFrame` wraps every page in `DensityScope compact`. 4, 5 and 6 are
per tab and belong to the tab threads. The shell has no workaround left: no Tessera class is touched.

## Overview header and tab bar

1. Switcher. Node shows a compact select at the top right whose value reads "Argus overview" and
   lists the overview plus the 8 boards. Argus shows a ghost "Switch project" button; its menu lists
   the 8 boards but not the overview, and the button never says where you are. Fix:
   `ProjectSwitcher.tsx` shows the current choice as its label and lists "Argus overview" first
   (Tessera Select size sm, or DropdownMenu with the current label).
2. Tab badges. Node shows a badge on every tab, also at zero once the tab was opened (grey "0"),
   with tones: Processes amber, Activity red, Notifications amber. Argus hides zero badges, so the
   badge disappears when you open the tab. Fix: `Tabs` with `hideZero={false}` and `badgeTone` per
   tab.
3. Spacing. Node puts the tab bar 12px under the title. Argus puts it 32px under the header panel.
   Fix: comes with the compact ContentHeader above.

Screenshots: `visual-audit/w1440-dark-overview-dashboards.png`, `visual-audit/x-ov-switcher.png`.

## Overview: Dashboards tab

1. Project card. Node card: a status dot and the project name in 16px bold, an amber warning pill
   with the need-you count at the top right, an uppercase status tag (RUNNING green, IDLE blue) with
   a clock icon and the age, then a row of three stat tiles with icons (agents, items, tests; the
   tests tile turns red when tests fail), flag chips ("6 failing" red, "2 decisions" amber), the
   branch "master" in mono with a grey filled Play button, the last run line, and a footer with
   three icon buttons (open board, send down, copy key) and the key in mono at the right. Argus
   card: a tinted header band with the name in uppercase and the key under it, plain tags "agents
   2", "items 9", "tests 15", a "9 need you" tag, an "Open board" text link and a "Send down"
   outline button. Fix: rebuild `ProjectCard.tsx` with `CardHeadline` (Status dot, Title, Badge
   warning count), `CardCounters` as three `StatTile` with an Icon and tone danger when failing,
   `MasterRow` with a filled secondary Button, and a footer ButtonRow of IconButton (external link,
   send, copy) plus mono Text for the key.
2. Card tone. Node keeps a neutral card surface and draws only the border in the attention tone (red
   when tests fail, amber when a decision waits, neutral otherwise). Argus tints the whole header
   band through `Card tone`. Fix: a Card border or edge tone that leaves the header neutral. This
   needs a new Tessera piece.
3. Grid. Node fits 6 cards per row at 1440 and 7 at 1600, all cards the same height with the footer
   pinned to the bottom; at 420 it still shows 2 per row. Argus fits 5 per row at 1440, heights
   follow the content, and 1 per row at 420. Fix: `ProjectGrid.tsx` Grid with a minimum column of
   about 180px, Card `fill`, and a Spacer above the footer.
4. Summary row. Node: "7 running" green pill, a warning icon and "30 need you" amber pill, a clock
   icon and "updated 55m ago", and "Send to all" with a send icon at the far right. Argus: the same
   pills without icons and "Send to all" right after the time. Fix: `DashboardTotals.tsx` Flex with
   justify between, Icon in the pills, Button with icon send.
5. Registered strip. Node: "Registered 8" then one neutral pill per project with a green dot. Argus:
   "Registered 8", a red "8 faults" tag, then bare red text "Archipelia, board closed" per project
   (4 lines at 420). The faults may come from shadow mode (presence streams reach Node only), but
   the look is wrong either way. Fix: `RegistryStrip.tsx` renders every project as a Tag with a
   Status dot and puts a fault inside its chip as a danger Tag with a warning icon, never as bare
   text.
6. Overview palette. The overview must stay on the basic palette, and the page root does (no
   `data-palette`). Inside the cards, though, the "Open board" link takes each project palette
   (purple on Archipelia and Bxel, orange on Brock, gold on RotP). Node shows every card in the same
   neutral look. Fix: `ProjectLink.tsx` uses the neutral link tone on the overview.
7. Order. Node lists Archipelia, Argus, Brock, Bxel, Overlord, RotP, Tessera, RotP Web (projects
   that need you first, then the rest). Argus sorts by name, so RotP Web comes before Tessera. Fix:
   port the Node sort to `ProjectGrid.tsx`.
8. Last run line. Node Bxel card: a dot, "exited 0 · 20h ago · [forensics] quit origin=..."
   truncated, and a "Log" link. Argus: "exited 19h ago" with no exit code, no message and no Log
   link. Fix: `RunningNow.tsx` shows the code, the truncated message and a Log Link.
9. Send to all and Send down. Node opens a small popover anchored under the button: a Decision /
   Note segmented control, "for all projects", a compact markdown toolbar, a 3 line textarea with a
   hint placeholder, "Send down" and "Ctrl+Enter sends". Argus opens a centred modal dialog titled
   "SEND DOWN TO ALL PROJECTS" with Write / Preview tabs, a 180px editor, Cancel and Send down. Fix:
   `SendDownDialog.tsx` becomes an anchored popover (Tessera Floating or Anchored) with a compact
   MarkdownEditor (see the editor item under Board: Activity).
10. Status word. Node: uppercase 11px tag "RUNNING" with the age and a clock icon at the right edge.
    Argus: lowercase "running" tag with the age right after it. Fix: Tag size xs uppercase and a
    Spacer before the age.
11. Sent down. Node: a full width bordered disclosure bar "Sent down (55)". Argus: a small chevron
    and text. Fix: `SentDown.tsx` Disclosure inside a PanelContainer.

Screenshots: `visual-audit/w1440-dark-overview-dashboards.png`,
`visual-audit/w420-dark-overview-dashboards.png`, `visual-audit/x-ov-send-all.png`.

## Overview: Processes tab

1. Columns off screen. The Argus DataTable is 2931px wide inside a 1390px box, so Age, Idle, Status,
   Command and the Kill button sit off screen to the right, and CPU is cut. Node fits all ten
   columns in 1440px. Fix: `ProcessTable.tsx` passes `fallbackColumns` with `fit` on the short
   columns, a fixed width on Rule, Process and Project, and `grow` on Command.
2. Summary. Node: one inline line, "90 processes memory 14 GB 1 need a look sampled 12s ago win32",
   numbers in bold. Argus: four boxed stat tiles (PROCESSES, MEMORY, NEED A LOOK, SAMPLED with "took
   30000 ms") and "win32" on its own line. Fix: `ProcessSummary.tsx` uses an inline `StatRow`.
3. Rule strip. Node: one chip per rule, name in bold, count and memory muted ("node-other 48 7.4
   GB"). Argus: a coloured StackedBar Node does not have, then plain chips "node-other 48, 8.5 GB".
   Fix: `RuleStrip.tsx` drops the StackedBar and renders Tag with bold name and muted figures.
4. Filters. Node: one row of pill toggles "All 90", "Needs a look 1", then "Rule" and a compact
   select. Argus: a boxed segmented control, then a full width "All rules" dropdown on its own row.
   Fix: `ProcessFilters.tsx` on one Flex row, Select size sm with auto width.
5. Rows. Node: process name in bold with the pid muted, the project as a link, an uppercase status
   tag (ORPHAN red, OK green), the command in mono truncated with a copy icon, a red outline Kill
   button or a lock icon for protected processes. Argus: process name muted, project plain text,
   memory in bold. Fix: `renderCell` in `ProcessTable.tsx` (`ProcessCell`, `StatusCell`, `KillCell`)
   with Text weight, Link, Tag uppercase tone, inline CodeBlock plus CopyButton, Button danger
   outline or Icon lock.
6. Narrow. At 420 Node keeps the table and scrolls it sideways with every column; Argus shows only
   Rule and part of Process, under 2 by 2 stat tiles that take 120px.

Screenshots: `visual-audit/w1440-dark-overview-processes.png`, `visual-audit/x-process-rules.png`,
`visual-audit/w420-dark-overview-processes.png`.

## Overview: Activity tab

1. Row layout. Node: one dense line per event (34px): a level icon, the age, the kind in uppercase
   mono coloured by level (blue info, amber warn, red error), the text, a project chip and "/
   thread" muted. Argus: a table with Kind, Event, Level and When columns, 60px rows, the kind as a
   bordered tag, the text in bold, project and thread on a second line, a level tag and the age at
   the right. Fix: `ActivityRow.tsx` as one `ListItemRow` line (Icon by level, ShortAge, Text mono
   uppercase toned, Text, Tag project, muted thread) and no table header in `ActivityFeed.tsx`.
2. Level. Node shows the level only through the icon and kind colour. Argus adds an "info" or "warn"
   tag on every row. Fix: drop the Level column.
3. Filter chips. Node: rounded pills, count muted, the selected one filled grey. Argus: square tags,
   the selected one outlined in white. Fix: `ChipFilter.tsx` renders selectable Tag pills with the
   filled selected look.

Screenshots: `visual-audit/w1440-dark-overview-activity.png`.

## Overview: Notifications tab

1. Rows. Node: one 52px row: bell icon, age, blue "ping" tag, bold title and the text inline,
   project chip, "/ coordinator", and "toast shown" muted under it. Argus: an 83px table row with
   the level, the title on its own line, the text, "toast shown", then When and Project columns at
   the right. Fix: `NoteRow.tsx` on one line like the activity row, no table header in
   `NoteFeed.tsx`.
2. Settings. Node: one line, "Notification settings INFO Toast Sound PING Toast Sound URGENT Toast
   Sound", each option a pill that is filled when on. Argus: a bordered panel titled NOTIFICATION
   SETTINGS with three columns of switches. Fix: `NotifySettings.tsx` inline Flex,
   `SettingToggle.tsx` as selectable Tag pills.
3. Test button. Node: a small 28px "Send a test notification". Argus: a 36px outline button. Fix:
   Button size sm in `TestPingBar.tsx`.

Screenshots: `visual-audit/w1440-dark-overview-notifications.png`.

## Board: header, registered strip and tab bar (all boards)

1. Tab bar position. Node puts the six tabs right under the header (about 147px from the top). Argus
   puts the Activity feed and a Work items list between the header and the tabs, so the tabs sit at
   about 1460px on RotP and 1700px on Brock, below the first screen at 1440x900 and far below it at 420.
   Fix: `BoardBody.tsx` keeps only the RegisteredStrip above the Tabs; OwnerFeed moves into the
   Activity tab; WorkItems goes away (Node has no such list; items live in Kanban).
2. Header content. Node: an uppercase eyebrow "COORDINATION BOARD", the board name, then on the same
   line the lead status (dot, lead name in bold, green "active" tag, "seen 15s ago", a chat icon
   link) and "board updated 14s ago"; at the right a project select ("RotP") and a kebab. Argus:
   "Back to Argus", the board name and a kebab in a gradient panel; no eyebrow, no lead line, no
   board updated time, no project switcher. Fix: `BoardPage.tsx` ContentHeader compact without
   backdrop, a caption eyebrow, the `strip` slot holding the lead line and a DateText, `actions`
   holding ProjectSwitcher and BoardMore; the "Back" link goes, since the switcher lists "Argus
   overview".
3. Tab badges. Node shows a count on every tab: Agents 0, Activity 2 (red), Kanban 4 (amber), Tests
   1 (amber), Builds 9, Worktrees 0. Argus shows only Tests. Fix: `BoardPage.tsx` gives every tab a
   badge and `badgeTone`, and `hideZero={false}`.
4. Registered chip. Node: "Registered 1" and a neutral chip with a green dot, the name, "+3 agents
   working" muted and an amber "1 stalled" tag. Argus: a red "1 fault" tag and a red chip whose
   words run together ("Brockboard closed+3 agents working1 stalled"). Fix: `RegisteredChip.tsx`
   puts a gap between ChipMark, the word and AgentText (Flex gap xs) and shows the fault as its own
   danger Tag.
5. Focus. Node has a page level "Focus: All threads" select under the tab bar on every tab. Argus
   has it only inside the Kanban toolbar as a full width select. Fix: move `FocusRow.tsx` to
   `BoardPage.tsx` under the Tabs, Select size sm with auto width.
6. Palette. The active tab underline follows the board palette in Argus (gold on RotP, orange on
   Brock) and stays white in Node. This is the intended palette rule, keep it.

Screenshots: `visual-audit/w1440-dark-rotp-agents.png`, `visual-audit/w1440-dark-brock-agents.png`,
`visual-audit/w420-dark-brock-kanban.png`.

Status (fix-board-shell): 1 to 5 done. The tab bar sits right under the registered strip (at 146px,
Node 147px). `BoardPage` holds `BoardHeader`, `RegisteredStrip`, `Tabs`, `BoardFocus` and
`BoardTabBody`; `BoardBody` and `WorkItems` are gone. The header is a plain Flex row (eyebrow, title,
lead status, board updated, `shell/ProjectSwitcher`, More), because Tessera `ContentHeader` has no
way to put an eyebrow over the title or to drop its panel padding and gradient. The registered chip
keeps Node's own fault look (red outline, red word) with a gap between mark, word and agent text.
`FocusRow` left the Kanban toolbar. Remaining gaps: the Tessera `Tabs` tab is 36px high against 40px
in Node, and the Select has no auto width (the Focus select sits in an inline Flex to shrink).
`ProjectSwitcher` also lists "Argus overview" first and ends with "(N running)" like Node; the
overview thread can reuse it.

## Board: Activity tab

1. Not built. The Argus Activity tab body says "This tab is not built yet." while the feed it should
   hold sits above the tab bar on every tab. Fix: `BoardPage.tsx` renders OwnerFeed when `tab ===
   "activity"` and removes it from BoardBody.
2. Decision card size. Node: a compact card with an amber left bar, an eyebrow "DECISION NEEDED Q-29
   coordinator I-88 7m" and a kebab, the context, the question in bold, the option buttons (the
   recommended one carries a RECOMMENDED suffix), and a one line editor (small toolbar, 3 line
   textarea with "Or write your own answer" as placeholder, Decide at the right, "Ctrl+Enter sends"
   under it). About 135px. Argus: a red header band "DECISION NEEDED", a meta line, the context, the
   question in small muted text, options in amber and green outline, a full MarkdownEditor with
   Write and Preview tabs and a 100px textarea, Decide under it. About 480px. Fix: `AskBody.tsx`,
   `AskForm.tsx`, `OptionButtons.tsx`: Card with a tone edge, the question as bold Text, secondary
   Buttons with the recommended suffix, and a compact editor. This needs a new Tessera piece.
   (MarkdownEditor needs a compact inline layout: toolbar and a short textarea, preview on demand.)
3. Header. Node: one row, "Activity 2 waiting on you" and the All / Needs you / Log segmented
   control at the right. Argus: a heading block "ACTIVITY / 2 waiting on you", then a second row
   "Activity view" with the segmented control. Fix: `OwnerFeed.tsx` PanelContainer header with the
   SegmentedControl in its actions.
4. Bundle to test. Node: one line with an amber left bar, a grey tag "bundle-14 · handed to you", "1
   test to run · title", "coordinator 7h" and an Open button. Argus: a brown header band BUNDLE TO
   TEST, a meta line, then the title and an "Open in Tests" link. Fix: `TestBody.tsx` as one
   ListItemRow.
5. Work log. Node: short ages ("26m"), the source in link blue (leads, portfolio, coordinator), the
   text, "audit" muted. Argus: long ages ("26 minutes ago") and a muted source. Fix: `LogList.tsx`
   with ShortAge and Link tone for the source.
6. Decision kebab. Node has a kebab on each decision card; Argus has none. Fix: a DropdownMenu in
   `CardMeta.tsx`.

Screenshots: `visual-audit/tab-rotp-activity.png`, `visual-audit/w1440-dark-rotp-activity.png`,
`visual-audit/x-activity-needs.png`, `visual-audit/x-activity-log.png`.

Status (fix-board-shell): 1, 3, 4, 5 and 6 done: `tabs/activity/ActivityTab` holds `OwnerFeed` in a
`shell/Panel` (title, count, view control on one row); the bundle and test cards are one line
(`TestLine`) with Open; the work log is a grid with short ages and a link coloured source; every
decision, question, manual step and notice card has a kebab (Archive, Delete, Copy id). 2 is half
done: the question is bold, the options are neutral buttons with a RECOMMENDED word and the card
head is one line; the amber left edge (Card edge tone) and the one line editor (compact
MarkdownEditor) wait for the next Tessera release.

## Board: Agents tab

1. Table versus cards. Node: a table with the headers STATUS, AGENT, MODEL, KIND, ITEM, BRANCH,
   TIME, NOW, LAST MESSAGE, NOTE and one 45px row per agent. Argus: 63px cards with no headers: name
   and id, a blue "now" line, the last message, then status, model, item and time at the right.
   Branch and Note columns are missing. Fix: `AgentList.tsx` and `AgentRow.tsx` as a Tessera Table
   (or DataTable with fixed columns) with those ten columns plus actions.
2. Row actions. Node: a pencil (edit note), a red stop square and a kebab (Open and edit, Edit note,
   Ask to stop, Archive, Delete, Copy id). Argus: only a stop button. Fix: `AgentActionButtons.tsx`
   adds IconButton edit and a DropdownMenu kebab with those entries.
3. Panel and lead line. Node wraps the list in a panel whose header holds "Agents", "working 0 ·
   waiting 0", the lead dot and note, "seen 12m" and a chat icon. Argus prints these as loose lines
   with the word "idle" or "working" and an underlined "Open the lead chat" link. Fix:
   `AgentsHeader.tsx` and `LeadLine.tsx` inside a PanelContainer header, a Status dot without the
   word, IconButton chat.
4. Status column. Node: first column, a dot and a tag ("working" green, "done" grey). Argus: status
   text at the right ("working 17 minutes ago"). Fix: `AgentStatus.tsx` as the first column with Tag
   and Status dot.
5. Model tones. Node: sonnet green, haiku blue, opus grey with a dot. Argus: sonnet takes the board
   accent (gold on RotP, orange on Brock). Fix: `AgentModel.tsx` fixed Tag colours per family; the
   palette primary is not a status tone.
6. Finished agents. Node: "Show finished (91)" under the table with the hint "Finished agents and
   runs stay as history; archived agents are in Trash."; opening it adds a FINISHED section in the
   same table (grey "done" rows, a branch icon where a worktree was kept). Argus: a Running / Done /
   All segmented control that swaps the list, an extra "Archive finished" button, and no hint.
   Counts differ: 91 against 85 on RotP, 222 against 219 on Brock (Node counts runs too). Fix:
   `AgentsToolbar.tsx` disclosure with the Node count rule and hint.
7. Search. Argus adds a "Search name, task, item or note" input that Node does not have. Fix: remove
   it from `AgentsToolbar.tsx`.
8. Time cell. Node: "18m" over "90 · 114". Argus: "18m" over "90 turns, 114 tools". Fix:
   `AgentTime.tsx` uses the Node format.
9. Empty. Node keeps the table headers and centres "No sub-agent is working right now." in the
   table. Argus shows the sentence alone.

Screenshots: `visual-audit/tab-brock-agents.png`, `visual-audit/tab-argus-agents.png`,
`visual-audit/tab-rotp-agents.png`, `visual-audit/x-agents-finished.png`,
`visual-audit/x-agent-kebab.png`.

## Board: Kanban tab

1. Toolbar layout is broken. Argus puts the search box in the left half, stacks Threads, Any status
   and Any subject as full width selects in the right half, the three toggles under them, then a
   full width "All threads" select. Node: one bordered panel; row one holds the search (220px), the
   Threads dropdown, the status and subject selects and three checkboxes (needs you, conflict,
   archived); row two holds Group by and Order segmented controls, "Show done (35)" and "4 open /
   39" at the right. Fix: `KanbanToolbar.tsx` and `FilterRow.tsx` as a Toolbar in a PanelContainer,
   one wrapping Flex row, Select size sm auto width, Checkbox instead of Toggle, Focus moved out
   (see the board header).
2. Columns. Node: the three columns share the full width. Argus: fixed 290px columns, so the right
   40% of the page is empty. Fix: `KanbanColumns.tsx` columns that grow to fill the row. This needs
   a new Tessera piece. (KanbanBoard has no fill option; its columns are fixed width in a flex row.)
3. Card. Node: checkbox, mono id, tags, and a kebab at the far right; title in 15px bold; footer
   with subject, thread, a criteria progress bar with "0/4", and the age at the far right. Argus: a
   drag handle, the kebab right after the tags, no progress bar, the age inline after the thread.
   Fix: `ItemCard.tsx` and `CardFoot.tsx` with a Spacer before the kebab and before the age,
   ProgressBar for the criteria, no visible drag handle (Node drags the whole card).
4. Needs you card. Node gives I-88 a red left edge. Argus has none. Fix: the Card edge tone from the
   overview.
5. Add item. Node: an inline form in the column, full width "New work item title" input, the thread
   select set to the column thread, Add and Cancel text buttons, a dashed border. Argus: the title
   input collapses to a 20px square, the thread select reads "Select..." with nothing chosen, and
   plus and cross icon buttons. Fix: `ColumnAddForm.tsx` TextInput that grows, Select with a default
   value, Buttons "Add" and "Cancel".
6. Card menu. Node: a header "I-79", Edit, Move to status with an inline select, Reassign to thread
   with an inline select, Archive, Delete in red, Copy id. Argus: Open details, Copy id, Move to
   status and Reassign to thread as submenus, Archive, Delete; no Edit. Fix: `CardMenu.tsx` in the
   Node order with Edit. This needs a new Tessera piece. (a menu row that holds a Select, if
   submenus are not accepted.)
7. Column header. Node: "DOING 2 [2 quiet]" and a plus icon. Argus: the same plus a "+ Add" text
   button and a collapse chevron Node does not have. Fix: `ColumnAdd.tsx` as IconButton plus, no
   collapse.
8. Words. Empty column: Node "none", Argus "No cards". Counter: Node "4 open / 39", Argus "4 open /
   39 · 3 quiet".
9. Threads filter. Node opens a small checklist under the button; Argus opens a full width list box
   over the toolbar. Same entries. Fix: Select size sm with a popup as wide as its content.

Screenshots: `visual-audit/tab-rotp-kanban.png`, `visual-audit/tab-argus-kanban.png`,
`visual-audit/tab-brock-kanban.png`, `visual-audit/x-card-kebab.png`, `visual-audit/x-add-item.png`,
`visual-audit/x-threads-filter.png`.

## Board: Tests tab

1. Toolbar is broken the same way as Kanban (search left, selects stacked right), and Node "Hide
   passed" (a link) becomes "Show passed" with a switch that is on. Fix: `TestsToolbar.tsx` and
   `FilterRow.tsx` like Kanban, a link style Button "Hide passed".
2. Test row. Node: one 48px row, mono id, the title in bold with the item and thread chips under it,
   the four result buttons (pass green, fail red, partial amber, blocked blue) and a kebab at the
   right. Argus: the title in regular weight, chips on a second line, the result buttons on a third
   line, all green tinted with Pass filled. Fix: `TestRow.tsx` on one Flex row, `OutcomeButtons.tsx`
   with a tone per outcome (success, danger, warning, info).
3. Bundle header. Node: one row, chevron, bundle name in bold and its subtitle, then at the right
   "ready" green, "handed to you" grey filled, "coordinator", "3 pass · 2 partial", a green and
   amber progress bar, and Unhand. Argus: the tags right after the name ("READY" uppercase), no
   progress bar, Unhand on its own line further down. Fix: `BundleHead.tsx` with a Spacer before the
   tags, `BundleProgress.tsx` as a 140px StackedBar, `HandButton.tsx` in the head.
4. Launch command. Node: a full width mono field with Copy and Play at the right, the run status
   after them, "run in X:/brock-example" and a "from launcher" chip with a link icon. Argus: inline
   mono text with a copy icon, an amber Play, and "from launcher" as an amber link. Fix:
   `LaunchCommand.tsx` as CopyValue full width plus a Play Button, "from" as a Tag with Icon link.
5. Terminal. Node: a bordered box with "Terminal", the last line, and "Full log" at the right.
   Argus: no box and no Full log. Fix: `RunTerminal.tsx` inside a PanelContainer with a "Full log"
   Link.
6. Done rows. Node: a row per test with an amber left bar for partial, the outcome tag, "you · 18h
   ago" and a blue "coordinator notified" tag at the right, the note on one truncated line. Argus:
   "Partial coordinator / 40 minutes ago", the note in full, and a "Run again" text button on every
   row (Node keeps it in the kebab). Fix: `DoneList.tsx` rows like the open rows, RerunButton inside
   TestMenu.
7. Thread chip. Node shows the thread id ("coordinator"); Argus shows the session name ("RotP",
   "Brock"). Fix: `TestChips.tsx` uses the thread id.
8. Show closed. Node: a link "Show closed (12)" at the left. Argus: a bold label with a switch at
   the far right. Fix: `ClosedSection.tsx` link style disclosure.

Screenshots: `visual-audit/tab-rotp-tests.png`, `visual-audit/tab-brock-tests.png`,
`visual-audit/tab-argus-tests.png`.

## Board: Builds tab

1. Command row. Node: a full width mono field, Copy and a white outline Play at the right, then the
   run status ("exited 0 · 48h ago · message") and a "Log" link at the far right. Argus: inline mono
   text with a copy icon, an amber Play, "exited 0 Yesterday message" and a green outline "Log"
   button. Fix: `LauncherCommand.tsx` CopyValue plus Button, `RunRow.tsx` with ShortAge and a Log
   Link.
2. Bundle link. Node: a chip with a link icon "for bundle bundle-7" and a separate "closed" tag.
   Argus: an amber underlined link "for bundle bundle-7 (ready)". Fix: `BundleLink.tsx` as Tag with
   Icon link and a status Tag.
3. Head. Node: "set by coordinator · 3d ago" right aligned. Argus: inline after the tags, "2 days
   ago". Fix: `LauncherHead.tsx` Spacer and ShortAge.
4. Current launcher. Node gives "master current" a green edge. Argus does not. Fix: the Card edge
   tone.
5. Panel. Node wraps the list in a "Builds 9 to start" panel. Argus prints the header line outside a
   panel. Fix: `BuildsHeader.tsx` as a PanelContainer header.
6. Terminal box with "Full log", as on the Tests tab.
7. Master tag. Node neutral grey; Argus amber from the palette. Fix: neutral Tag.

Screenshots: `visual-audit/tab-rotp-builds.png`, `visual-audit/tab-brock-builds.png`,
`visual-audit/tab-argus-builds.png`.

## Board: Worktrees tab

1. Columns off screen. The Argus DataTable is 2365px wide in a 1390px box: Last commit is cut,
   Request and Actions (Ask the thread, Remove) are off screen. Fix: `WorktreeTable.tsx`
   `fallbackColumns` with `grow` on Path and Last commit, fixed Branch and `fit` on the rest. Node
   also clips "Ask the thre..." at 1440; do not copy that.
2. Header. Node: a panel header "Worktrees 21 worktrees · 0 need a decision · scanned 2m ago · RotP
   has 21 worktrees...", an explainer paragraph, and the All / Need a decision / Leftovers segmented
   control at the right. Argus: the summary as loose text with the warning in orange, a "Show"
   segmented control on the next line, no explainer. Fix: `WorktreeSummary.tsx` and
   `WorktreeToolbar.tsx` in a PanelContainer header.
3. Status tags. Node: lowercase tags (kept grey, active green, leftover dashed, partial amber,
   locked grey). Argus: uppercase pills (KEPT blue, ACTIVE green, LEFTOVER outline). Fix:
   `StatusCell.tsx` Tag lowercase with the Node tones.
4. Branch. Node: bold mono text that wraps, the short sha under it. Argus: each branch inside a
   bordered input-like box. Fix: `BranchCell.tsx` plain mono Text.
5. Touched. Node: "5d ago", "last changed 15h ago". Argus: durations "4d 18h", "last changed 15h
   21m". Fix: `TouchedCell.tsx` ShortAge.
6. Table chrome. Argus shows the DataTable footer "No sorting or grouping ... 21 rows" with a gear.
   Node has none. Fix: hide the footer. This needs a new Tessera piece. (DataTable has no prop to
   hide its view footer.)
7. Copy. Node: a "Copy" text button after the path. Argus: a copy icon. Fix: CopyButton with its
   label in `PathCell.tsx`.

Screenshots: `visual-audit/tab-rotp-worktrees.png`, `visual-audit/tab-argus-worktrees.png`,
`visual-audit/tab-brock-worktrees.png`.

## Drawers: what every drawer shares

1. Width. Node drawers are 760px at 1440 (more than half the page) and full width at 420. Argus
   drawers are 384px (`size="md"`) at 1440 and 86% at 420. Fix: `size="lg"` is the closest (640px);
   exact parity needs a 760px size and full width under 480px. This needs a new Tessera piece.
2. Header. Node: an eyebrow line (mono id and the status tag, or for a test "T-133 · BROCK 0.106 /
   TESSERA 0.72 ADOPTION"), the full title wrapping in 20px bold, and the tools as icons in the
   header row (copy link, previous, next, Expand, archive, a red delete, close). Argus: the Drawer
   WindowHeader title on one line, cut with an ellipsis, in the palette accent colour and in
   uppercase for tests and agents; the tools sit in a second row (item, thread) or in a footer
   (test). Fix: a Drawer record header with an eyebrow slot, a wrapping title and a header actions
   slot, used by every drawer. This needs a new Tessera piece.
3. Archive and delete. Node: an archive icon and a red trash icon. Argus: green "Archive" and red
   "Delete" outline text buttons. Fix: IconButton in the new header.

Screenshots: `visual-audit/w1440-dark-rotp-drawer-item.png`,
`visual-audit/w1440-dark-rotp-drawer-test.png`, `visual-audit/w420-dark-rotp-drawer-test.png`.

## Drawer: work item

1. Facts. Node: one row "Status [Doing] Thread [RotP] Subject coordination" with compact selects,
   then "rev 5, updated by coordinator 8h ago". Argus: a status tag, then the status select and the
   thread select full width on separate lines, then the subject on its own line. Fix:
   `DrawerFacts.tsx` one Flex row with inline labelled Select size sm.
2. Sections. Node: uppercase titles with a middle dot ("ACCEPTANCE CRITERIA · 0/0") and a rule
   between sections; empty lines centred and muted ("None entered.", "No to-do on this item.").
   Argus: comma titles, no rules, empty lines left aligned. Fix: `DrawerSection.tsx` with
   SectionHeader and Divider.
3. Add rows. Node: an input and a small "Add" button. Argus: "Add criterion" and "Add to-do". Fix:
   `AddLine.tsx` label "Add".
4. Questions. Node: a bordered question card (id, "question" tag, meta, "answered" tag, kebab,
   context, the question in bold) and replies indented with a left rule. Argus: the same facts as a
   flat list. Fix: `EntryBlock.tsx` and `EntryReply.tsx` as a Card with an indented reply list.

Screenshots: `visual-audit/w1440-dark-rotp-drawer-item.png`,
`visual-audit/w1440-dark-brock-drawer-item.png`, `visual-audit/x-item-edit.png`.

## Drawer: test

1. Missing controls. Node has a Mode select ("manual"), an Item select and a "Covers Brock 0.106 /
   Tessera 0.72 adoption" link. Argus shows only tags. Fix: `TestFacts.tsx` with the two Selects and
   the link.
2. Expected. Node: two cards side by side, green PROVES IT WORKS and red PROVES IT IS BROKEN. Argus:
   two stacked plain lists with coloured labels. Fix: `ExpectedLists.tsx` Grid of two columns of
   Card tone success and danger.
3. Steps. Node: numbered circles with a rule between steps. Argus: a plain ordered list in bold.
   Fix: `SetupList.tsx` with Stepper style numbers and Divider.
4. Result. Node: a bordered card "RESULT [to run]" with "Not run yet." centred. Argus: plain text.
   Fix: `ResultBlock.tsx` in a Card.
5. Run buttons. Node: pass green, fail red, partial amber, blocked blue. Argus: all green. Fix:
   tones in `OutcomeButtons.tsx`.
6. Footer. Argus puts Previous, Next, Copy link, Expand, Edit, Archive and Delete in a footer, and
   the item drawer puts its tools at the top: two layouts for one job. Node has them in the header
   of every drawer. Fix: `DrawerActions.tsx` moves into the new header.

Screenshots: `visual-audit/w1440-dark-rotp-drawer-test.png`,
`visual-audit/w1440-dark-brock-drawer-test.png`.

## Drawer: agent

1. Missing Where section. Node: WHERE with Model, Kind, Branch ("add branch") and Worktree ("add
   worktree") as inline editable fields. Argus: none. Fix: a WHERE DrawerSection in
   `AgentDrawerBody.tsx` with InlineEdit fields.
2. Missing Item select. Node: "Item [I-49 Archipelia B3 ...]" select and "Thread coordinator".
   Argus: a read only facts box. Fix: `AgentFacts.tsx` Select for the item.
3. Order and timeline. Node: Task, Note, Result, Where, Timeline ("Started 24/10/2026, 02:22:57 (3d
   ago)"). Argus: a facts box first (thread, item, agent id, started, updated, finished, duration)
   without seconds or the relative age, then Task, Note, Result. Fix: Node order, a Timeline section
   with seconds and the age.
4. Title. Node: "review-server-packaging" in bold with the eyebrow "A-99 done opus research". Argus:
   "REVIEW-SERVER-PACKAGING A.." uppercase, cut, accent colour. Fix: the new Drawer header.

Screenshots: `visual-audit/w1440-dark-brock-drawer-agent.png`,
`visual-audit/w1440-dark-rotp-drawer-agent.png`.

## Drawer: thread

1. Open work rows are broken. Argus wraps the item id over two lines ("I-" then "72") and wraps the
   titles, because the id column is too narrow in the 384px drawer. Node: full width one line mini
   cards (id, status tag, title). Fix: `ThreadWork.tsx` as ListItemRow with a nowrap mono id.
2. Header. Node: eyebrow "coordinator [active] idle [coord]", title "Brock", tools: copy link,
   previous, next, Expand, chat icon, copy, kebab, close. Argus: "BROCK coordinator" in accent
   uppercase, then a row with Expand, an underlined "Open the chat" link and two unlabelled copy
   icons; no previous, next or kebab. Fix: the new Drawer header with the thread kebab from
   `ThreadTools.tsx`.
3. Subject. Node: "Subject: add subject" as an italic dashed link next to "seen 59s ago". Argus:
   plain text. Fix: InlineEdit in `ThreadFacts.tsx`.

Screenshots: `visual-audit/w1440-dark-brock-drawer-thread.png`,
`visual-audit/w1440-dark-rotp-drawer-thread.png`.

## Menus, popovers and editors that only open

1. Board More menu. Node: Trash (0), Resources and locks (0 held), Legend, Show hidden (5), Purge
   threads and records (red). Argus: Trash (0) and Purge only. Fix: `BoardMore.tsx` adds Resources
   and locks, Legend and Show hidden with their views.
2. Trash view. Node replaces the tab body with a "Trash 0 in trash [Back to the board]" panel and
   "Trash is empty.". Argus shows its trash under the feed, below the first screen. Fix: comes with
   the BoardBody fix.
3. Send down on a card. Node opens the same anchored popover as Send to all, under the card send
   icon. Argus opens the centred dialog. Fix: as Send to all.
4. Agent kebab and card kebab: see the Agents and Kanban sections.
5. Project switcher on boards: Node has it on every board, Argus only on the overview (see the board
   header).

Screenshots: `visual-audit/x-board-more.png`, `visual-audit/x-board-trash.png`,
`visual-audit/x-ov-send-down.png`, `visual-audit/x-ov-switcher.png`.

Status (fix-board-shell): 1 done: the More menu lists Trash, Resources and locks (N held, a dialog
with the lock table), Legend (a dialog), Show hidden (N, stored as `coord.hidden`) and Purge. The
Agents tab thread must read `coord.hidden` (`HIDDEN_KEY` in `BoardMore.constants.ts`) to show the
hidden threads. 2 and 5 done. 3 belongs to the overview and kanban threads (needs the compact
editor). 4 is the tab threads'.

## Narrow window (420px)

1. First screen. On every board Node shows header, tabs, Focus and the first cards of the tab; Argus
   shows the header panel, the registered faults and the Activity feed, and no tab at all. Fix: the
   BoardBody fix.
2. Overview cards. Node: 2 cards per row, 4 visible. Argus: 1 per row, 1.5 visible. Fix: the
   ProjectGrid minimum column.
3. Header. Node keeps the title and the select on one 50px row. Argus: a 64px panel inside a 24px
   inset.
4. Tab bar. Argus shows a scroll chevron at the right of the tab bar; Node clips the last tab. Argus
   is better here; keep it.

Screenshots: `visual-audit/w420-dark-overview-dashboards.png`,
`visual-audit/w420-dark-rotp-agents.png`, `visual-audit/w420-dark-brock-kanban.png`,
`visual-audit/w420-dark-overview-processes.png`.

Status (fix-board-shell): 1 and 3 done on boards (header, tabs, Focus and the first rows fit the
first screen at 420px; the header folds like Node: title, switcher and kebab on one row, lead line
under it). 2 belongs to the overview thread. 4 kept.

## Empty, busy and not found states

1. Empty tests (Argus board): same sentence on both. Argus wraps it in the broken toolbar; Node in
   its compact panel.
2. Empty agents (RotP): Node keeps the table frame and headers; Argus shows the sentence alone (see
   Agents).
3. Empty Kanban column: "none" against "No cards" (see Kanban).
4. Unknown board: both serve the same "Not found / There is no board for ..." page. No difference.
5. Busy: both pages load in under 120ms on this machine, so neither loading state could be captured.
   Node shows "Connecting to the dashboard..." in its status line while the stream opens; check that
   Argus says the same when the server is slow.

Screenshots: `visual-audit/tab-argus-tests.png`, `visual-audit/tab-rotp-agents.png`,
`visual-audit/x-unknown-board.png`, `visual-audit/x-loading-overview.png`.

Status (fix-board-shell): 5 done: the board header says "connecting" until the stream has the
board, then "board updated Xs ago", and "dashboard unreachable" when it is down. 1 to 3 follow the
tab threads. 4: the Vite dev server shows an empty board for an unknown key; the Rust server
returns the Not found page, which is the real behaviour.

## Palette check

1. Overview: no `data-palette`, primary #8a8a96 (basic). Correct.
2. rotp: `rotp`, primary #c8a84e (gold). Correct.
3. brock: `brock`, primary #f0862b (orange). Correct.
4. argus: basic. Correct (Argus wears the basic palette).
5. archipelia: `archipelia`, primary #7c4dff. Correct.
6. pixelarttool (Bxel): `bxel`, primary #7b3fe4. Correct.
7. tessera, portfolio (Overlord), rotp-web: basic. rotp-web could use the RotP palette; that is the
   owner's call.
8. Switching board to board inside the page (client navigation) changes the palette each time, and
   going back to the overview clears it. Correct.
9. Leaks: on the overview the "Open board" links use each project palette; on boards the palette
   primary paints the sonnet model tag, the master tag and the Play buttons, which are status or
   neutral marks in Node. The palette should drive the accent only (active tab, links, primary
   buttons).
10. Node has one palette for every board, so this is an intended difference, not a parity gap.

Status (fix-board-shell): the shell, the header, the strip, the focus bar, the Activity tab and the
More menu use no palette colour except the active tab and focus ring. The leaks in item 9 sit in
other folders: `tabs/agents/model/model.constants.ts` (sonnet is `primary`),
`tabs/builds/sub-components/LauncherChips.tsx` (master), `tabs/tests/sub-components/BundleHead.tsx`
and `FactTags.tsx` (handed to you), `tabs/tests/tests.constants.ts` and
`tabs/worktrees/model/model.constants.ts` (`primary` tones). The tab threads move them to neutral
or status tones.

## Data differences seen on screen

These are not layout, but the owner sees them side by side. Each needs a check on the server side.

1. Registered strip: Argus flags every board "board closed" because the presence streams go to Node
   while Argus runs in shadow mode. Check again after the switch-over.
2. Agents: finished counts differ (RotP 91 against 85, Brock 222 against 219) because Node counts
   runs with agents.
3. Work item I-86: Node shows "Tests · 0/0 passing"; Argus shows "1/3 passing" and lists T-125 and
   T-126, which are moved tests.
4. Notifications: Argus shows "22m ago" for notifications Node dates 1h, 4h, 5h and 7h ago; Argus
   uses its own receive time.
5. Brock tests bundle: Node shows the last run R-3 "exited 0 · 25m ago"; Argus shows R-2 "exited 0
   18 hours ago".
6. RotP builds: Node marks bundle-7 to bundle-13 "closed"; Argus says "(ready)".
7. Processes: Argus samples its own process list (different totals, and rules Node does not have:
   electron-dev, python, project-build, game-engine).
8. Overview Activity: the two servers keep their own logs, so counts and rows differ (128 against
   134).

## New Tessera pieces

1. A light theme for the basic palette and each project palette, switched by `prefers-color-scheme`
   (Tessera ships dark only). Dark only by owner rule: not a target.
2. A Drawer record header: an eyebrow slot, a title that wraps, and a header actions slot for icon
   tools; plus a 760px size and full width under 480px.
3. A Card edge tone: a coloured border or left edge in a status tone that leaves the header neutral
   (project cards, needs-you cards, current launcher, decision cards, partial test rows).
4. A compact inline layout for MarkdownEditor: toolbar and a 3 line textarea, preview only on
   demand, for replies and send down.
5. KanbanBoard columns that grow to fill the row.
6. A DataTable prop to hide the view footer (sort and group summary, row count, gear).
7. Optional: a DropdownMenu row that holds a Select, for Move to status and Reassign to thread, if
   submenus are not accepted as parity.

## Screenshots

The 134 side by side images are in `X:/argus/docs/visual-audit/` (Node on the left or on top, Argus
on the right or below; a magenta bar separates them). The raw screenshots, the text of each page and
the capture scripts are in the session scratchpad folder `visual`. Name pattern:
`<viewport>-<theme>-<screen>.png` for whole pages, `tab-<board>-<tab>.png` for a tab from the tab
bar down, `x-<what>.png` for menus, popovers and states.

- `X:/argus/docs/visual-audit/tab-argus-activity.png`
- `X:/argus/docs/visual-audit/tab-argus-agents.png`
- `X:/argus/docs/visual-audit/tab-argus-builds.png`
- `X:/argus/docs/visual-audit/tab-argus-kanban.png`
- `X:/argus/docs/visual-audit/tab-argus-tests.png`
- `X:/argus/docs/visual-audit/tab-argus-worktrees.png`
- `X:/argus/docs/visual-audit/tab-brock-activity.png`
- `X:/argus/docs/visual-audit/tab-brock-agents.png`
- `X:/argus/docs/visual-audit/tab-brock-builds.png`
- `X:/argus/docs/visual-audit/tab-brock-kanban.png`
- `X:/argus/docs/visual-audit/tab-brock-tests.png`
- `X:/argus/docs/visual-audit/tab-brock-worktrees.png`
- `X:/argus/docs/visual-audit/tab-rotp-activity.png`
- `X:/argus/docs/visual-audit/tab-rotp-agents.png`
- `X:/argus/docs/visual-audit/tab-rotp-builds.png`
- `X:/argus/docs/visual-audit/tab-rotp-kanban.png`
- `X:/argus/docs/visual-audit/tab-rotp-tests.png`
- `X:/argus/docs/visual-audit/tab-rotp-worktrees.png`
- `X:/argus/docs/visual-audit/w1440-dark-argus-activity.png`
- `X:/argus/docs/visual-audit/w1440-dark-argus-agents.png`
- `X:/argus/docs/visual-audit/w1440-dark-argus-builds.png`
- `X:/argus/docs/visual-audit/w1440-dark-argus-drawer-agent.png`
- `X:/argus/docs/visual-audit/w1440-dark-argus-drawer-item.png`
- `X:/argus/docs/visual-audit/w1440-dark-argus-drawer-thread.png`
- `X:/argus/docs/visual-audit/w1440-dark-argus-kanban.png`
- `X:/argus/docs/visual-audit/w1440-dark-argus-tests.png`
- `X:/argus/docs/visual-audit/w1440-dark-argus-worktrees.png`
- `X:/argus/docs/visual-audit/w1440-dark-brock-activity.png`
- `X:/argus/docs/visual-audit/w1440-dark-brock-agents.png`
- `X:/argus/docs/visual-audit/w1440-dark-brock-builds.png`
- `X:/argus/docs/visual-audit/w1440-dark-brock-drawer-agent.png`
- `X:/argus/docs/visual-audit/w1440-dark-brock-drawer-item.png`
- `X:/argus/docs/visual-audit/w1440-dark-brock-drawer-test.png`
- `X:/argus/docs/visual-audit/w1440-dark-brock-drawer-thread.png`
- `X:/argus/docs/visual-audit/w1440-dark-brock-kanban.png`
- `X:/argus/docs/visual-audit/w1440-dark-brock-tests.png`
- `X:/argus/docs/visual-audit/w1440-dark-brock-worktrees.png`
- `X:/argus/docs/visual-audit/w1440-dark-overview-activity.png`
- `X:/argus/docs/visual-audit/w1440-dark-overview-dashboards.png`
- `X:/argus/docs/visual-audit/w1440-dark-overview-notifications.png`
- `X:/argus/docs/visual-audit/w1440-dark-overview-processes.png`
- `X:/argus/docs/visual-audit/w1440-dark-rotp-activity.png`
- `X:/argus/docs/visual-audit/w1440-dark-rotp-agents.png`
- `X:/argus/docs/visual-audit/w1440-dark-rotp-builds.png`
- `X:/argus/docs/visual-audit/w1440-dark-rotp-drawer-agent.png`
- `X:/argus/docs/visual-audit/w1440-dark-rotp-drawer-item.png`
- `X:/argus/docs/visual-audit/w1440-dark-rotp-drawer-test.png`
- `X:/argus/docs/visual-audit/w1440-dark-rotp-drawer-thread.png`
- `X:/argus/docs/visual-audit/w1440-dark-rotp-kanban.png`
- `X:/argus/docs/visual-audit/w1440-dark-rotp-tests.png`
- `X:/argus/docs/visual-audit/w1440-dark-rotp-worktrees.png`
- `X:/argus/docs/visual-audit/w1440-light-argus-activity.png`
- `X:/argus/docs/visual-audit/w1440-light-argus-agents.png`
- `X:/argus/docs/visual-audit/w1440-light-argus-builds.png`
- `X:/argus/docs/visual-audit/w1440-light-argus-drawer-agent.png`
- `X:/argus/docs/visual-audit/w1440-light-argus-drawer-item.png`
- `X:/argus/docs/visual-audit/w1440-light-argus-drawer-thread.png`
- `X:/argus/docs/visual-audit/w1440-light-argus-kanban.png`
- `X:/argus/docs/visual-audit/w1440-light-argus-tests.png`
- `X:/argus/docs/visual-audit/w1440-light-argus-worktrees.png`
- `X:/argus/docs/visual-audit/w1440-light-brock-activity.png`
- `X:/argus/docs/visual-audit/w1440-light-brock-agents.png`
- `X:/argus/docs/visual-audit/w1440-light-brock-builds.png`
- `X:/argus/docs/visual-audit/w1440-light-brock-drawer-agent.png`
- `X:/argus/docs/visual-audit/w1440-light-brock-drawer-item.png`
- `X:/argus/docs/visual-audit/w1440-light-brock-drawer-test.png`
- `X:/argus/docs/visual-audit/w1440-light-brock-drawer-thread.png`
- `X:/argus/docs/visual-audit/w1440-light-brock-kanban.png`
- `X:/argus/docs/visual-audit/w1440-light-brock-tests.png`
- `X:/argus/docs/visual-audit/w1440-light-brock-worktrees.png`
- `X:/argus/docs/visual-audit/w1440-light-overview-activity.png`
- `X:/argus/docs/visual-audit/w1440-light-overview-dashboards.png`
- `X:/argus/docs/visual-audit/w1440-light-overview-notifications.png`
- `X:/argus/docs/visual-audit/w1440-light-overview-processes.png`
- `X:/argus/docs/visual-audit/w1440-light-rotp-activity.png`
- `X:/argus/docs/visual-audit/w1440-light-rotp-agents.png`
- `X:/argus/docs/visual-audit/w1440-light-rotp-builds.png`
- `X:/argus/docs/visual-audit/w1440-light-rotp-drawer-agent.png`
- `X:/argus/docs/visual-audit/w1440-light-rotp-drawer-item.png`
- `X:/argus/docs/visual-audit/w1440-light-rotp-drawer-test.png`
- `X:/argus/docs/visual-audit/w1440-light-rotp-drawer-thread.png`
- `X:/argus/docs/visual-audit/w1440-light-rotp-kanban.png`
- `X:/argus/docs/visual-audit/w1440-light-rotp-tests.png`
- `X:/argus/docs/visual-audit/w1440-light-rotp-worktrees.png`
- `X:/argus/docs/visual-audit/w420-dark-argus-activity.png`
- `X:/argus/docs/visual-audit/w420-dark-argus-agents.png`
- `X:/argus/docs/visual-audit/w420-dark-argus-builds.png`
- `X:/argus/docs/visual-audit/w420-dark-argus-drawer-agent.png`
- `X:/argus/docs/visual-audit/w420-dark-argus-drawer-item.png`
- `X:/argus/docs/visual-audit/w420-dark-argus-drawer-thread.png`
- `X:/argus/docs/visual-audit/w420-dark-argus-kanban.png`
- `X:/argus/docs/visual-audit/w420-dark-argus-tests.png`
- `X:/argus/docs/visual-audit/w420-dark-argus-worktrees.png`
- `X:/argus/docs/visual-audit/w420-dark-brock-activity.png`
- `X:/argus/docs/visual-audit/w420-dark-brock-agents.png`
- `X:/argus/docs/visual-audit/w420-dark-brock-builds.png`
- `X:/argus/docs/visual-audit/w420-dark-brock-drawer-agent.png`
- `X:/argus/docs/visual-audit/w420-dark-brock-drawer-item.png`
- `X:/argus/docs/visual-audit/w420-dark-brock-drawer-test.png`
- `X:/argus/docs/visual-audit/w420-dark-brock-drawer-thread.png`
- `X:/argus/docs/visual-audit/w420-dark-brock-kanban.png`
- `X:/argus/docs/visual-audit/w420-dark-brock-tests.png`
- `X:/argus/docs/visual-audit/w420-dark-brock-worktrees.png`
- `X:/argus/docs/visual-audit/w420-dark-overview-activity.png`
- `X:/argus/docs/visual-audit/w420-dark-overview-dashboards.png`
- `X:/argus/docs/visual-audit/w420-dark-overview-notifications.png`
- `X:/argus/docs/visual-audit/w420-dark-overview-processes.png`
- `X:/argus/docs/visual-audit/w420-dark-rotp-activity.png`
- `X:/argus/docs/visual-audit/w420-dark-rotp-agents.png`
- `X:/argus/docs/visual-audit/w420-dark-rotp-builds.png`
- `X:/argus/docs/visual-audit/w420-dark-rotp-drawer-agent.png`
- `X:/argus/docs/visual-audit/w420-dark-rotp-drawer-item.png`
- `X:/argus/docs/visual-audit/w420-dark-rotp-drawer-test.png`
- `X:/argus/docs/visual-audit/w420-dark-rotp-drawer-thread.png`
- `X:/argus/docs/visual-audit/w420-dark-rotp-kanban.png`
- `X:/argus/docs/visual-audit/w420-dark-rotp-tests.png`
- `X:/argus/docs/visual-audit/w420-dark-rotp-worktrees.png`
- `X:/argus/docs/visual-audit/x-activity-log.png`
- `X:/argus/docs/visual-audit/x-activity-needs.png`
- `X:/argus/docs/visual-audit/x-add-item.png`
- `X:/argus/docs/visual-audit/x-agent-kebab.png`
- `X:/argus/docs/visual-audit/x-agents-finished.png`
- `X:/argus/docs/visual-audit/x-board-more.png`
- `X:/argus/docs/visual-audit/x-board-trash.png`
- `X:/argus/docs/visual-audit/x-card-kebab.png`
- `X:/argus/docs/visual-audit/x-item-edit.png`
- `X:/argus/docs/visual-audit/x-loading-board.png`
- `X:/argus/docs/visual-audit/x-loading-overview.png`
- `X:/argus/docs/visual-audit/x-ov-send-all.png`
- `X:/argus/docs/visual-audit/x-ov-send-down.png`
- `X:/argus/docs/visual-audit/x-ov-switcher.png`
- `X:/argus/docs/visual-audit/x-process-rules.png`
- `X:/argus/docs/visual-audit/x-threads-filter.png`
- `X:/argus/docs/visual-audit/x-unknown-board.png`
