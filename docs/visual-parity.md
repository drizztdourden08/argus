<!-- @layer docs @kind doc -->
# Visual parity: Node dashboard against Argus

Side by side screenshots of the Node dashboard (port 47615) and Argus in shadow mode (port 47616),
taken headless at 1440x900 and 420x900, dark and light, on the boards RotP, Argus and Brock. Each
section lists the differences, the most visible first, with the fix in Argus terms. A fix that needs
a piece Tessera does not have yet says so. The gallery `docs/visual-audit/index.html` shows every
pair with the same numbered list under it.

Waits on Tessera (kanban head row): a 16px IconButton, a 16px Text line, an outline TypeMark and a square image Avatar; until then the card head uses the xs sizes and a circle for lead initials.

## Re-audit 2026-10-27 (after the fix wave on main)

Argus main (Vite dev server, read only proxy to the Node dashboard) shot again against Node at
1440x900 and 420x900, dark only, on RotP, Argus and Brock, with every tab, drawer, menu and the
narrow views. Item numbers read section.item (the section title, then the number in its list); the
numbered differences 119 and up are global. The gallery `docs/visual-audit/index.html` shows the pairs
of the first round with this status list.

Close-out pass (same day, Tessera 0.84): every "open" entry was checked against Node again, a fresh
side by side sweep of every overview tab, every board tab, the main drawers and the menus at 1440 and
420 added items 144 to 160, and the ones that need no new Tessera piece are fixed. What is still open
is in the closing list "Still open after the close-out" at the end of the new differences. The old
audit sections further down are the first round, kept for the screenshots; where they disagree with
this section, this section is true.

### Page shell, theme and density (every screen)

1. dropped (dark only): light theme, dark only by owner rule.
2. fixed: board and overview titles start at 16px, the overview tab bar sits at 66px and the board tab
   bar at 128px, as Node (own plain headers, no ContentHeader).
3. fixed: activity rows 33px to 34px, notifications 51px to 52px, same as Node. Processes rows are
   the exception, see new item 119.
4. fixed: uppercase letter-spaced column and section headers with the middle dot, as Node.
5. fixed: one age rule (seconds, minutes, hours up to 47h, then days, all rounded, `formatAgo`, shown
   by the Age component) in every board and overview age. The last places that still used the Tessera
   relative text ("1 day ago") were the registered strip tooltips, the retired chip, the leftover
   delete text, the worktree request cell and the results history: they use Age now (item 146).
6. fixed: the activity event text is Text size="md" (the 13px step in Tessera 0.84), the same colour (#f0f0f2) as Node. Notification titles are bold in both.

### Overview header and tab bar

1. fixed: switcher reads "Argus overview" and lists the overview first, then the 8 boards.
2. fixed: every tab keeps its badge, grey zero after opening, amber and red tones as Node. The badge is solid (item 121).
3. fixed: the tab bar sits at 66px as Node. Each tab is now 40px high as Node (new item 122).

### Overview: Dashboards tab

1. fixed: card built as Node (dot, name, warning pill, uppercase status with clock, three stat
   tiles, chips, master row, footer icons and key). The tile icons differ, see new item 120.
2. fixed: card border carries the attention tone, header stays neutral.
3. fixed: 6 cards per row at 1440, 2 at 420, equal heights, footer pinned.
4. fixed: summary row with icons, Send to all at the far right.
5. fixed: registered strip: one chip per project with a dot, faults as chips with a warning icon
   (shape differs, see new item 133).
6. fixed: Open board link is an icon button, no palette colour on the overview.
7. fixed: same order as Node (Archipelia, Brock, Bxel, RotP, Tessera, Argus, Overlord, RotP Web).
8. fixed: last run line shows exited 0, age, message and a Log link. The status line is a Tessera FoldText (one line, ellipsis), with no one-off CSS.
9. fixed: Send to all opens an anchored popover with Decision and Note, compact editor and
   Ctrl+Enter hint. Layout details differ, see new item 140.
10. fixed: uppercase status tag with the age and clock at the right.
11. fixed: Sent down is a full width bordered disclosure bar.

### Overview: Processes tab

1. fixed: all ten columns fit in 1440px, Kill and lock visible.
2. fixed: one inline summary line with bold numbers.
3. fixed: rule chips with bold name and muted figures, Kill stale chip, no stacked bar.
4. fixed: pill toggles All and Needs a look, Rule select on the same row.
5. fixed: rows are 40px as Node, the project link is Link tone link (lavender, underlined) and the
   needs-a-look row draws only the left bar (DataTable rowState edge). See new item 119.
6. fixed: at 420 the table keeps its columns and scrolls sideways, as Node.

### Overview: Activity tab

1. fixed: one dense line per event: icon, short age, uppercase mono kind toned by level, text,
   project chip, muted thread.
2. fixed: no level column, no level tag.
3. fixed: chips are full pills with the filled grey selected state, as Node (item 123), checked again
   in the close-out: levels, kinds and projects all fill grey when selected.

### Overview: Notifications tab

1. fixed: one row per notification: bell, age, ping tag, bold title, text, project chip, "toast
   shown" under it.
2. fixed: settings line is inline as Node and a switched-on option is a filled grey pill (item 123),
   checked again in the close-out.
3. fixed: small Send a test notification button.

### Board: header, registered strip and tab bar (all boards)

1. fixed: tab bar directly under the registered strip (130px against 128px), first rows on the first
   screen at 1440 and 420.
2. fixed: eyebrow, name, lead status with active tag, seen, chat icon, board updated, board select
   and kebab. The title row starts 16px lower (new item 131) but the tab bar lands at the same
   height.
3. fixed: a badge on every tab with Node tones and counts (solid badge, item 121).
4. fixed: the fault is a rounded outlined pill with an icon, the name in the text colour and bold, the
   reason in red (items 133 and 154). The faulted project name used to be red as a whole.
5. fixed: page level Focus select under the tab bar on every tab.
6. fixed: palette kept: active tab underline follows the board palette (intended).

### Board: Activity tab

1. fixed: Activity tab holds the feed in a panel, view control in the panel header.
2. fixed: decision card has the Node eyebrow (item as plain mono text, kebab at the right), bold
   question, RECOMMENDED options, the amber left edge (item 127, fixed for ask cards), a 3 row box
   with the prompt as placeholder. Decide (and Answer) now sit at the right of the editor toolbar
   (MarkdownEditor toolbarEnd); Dismiss stays under the box. Covered by tests, not seen on screen
   because no board had a pending decision when the round was shot.
3. fixed: one header row with the All, Needs you and Log control (selected tone differs, see new
   item 124).
4. fixed: bundle to test is one line with tag, title, lead, age and Open. The amber left edge
   is missing, see new item 127.
5. fixed: work log with short ages and link coloured source.
6. fixed: kebab on every decision card.

### Board: Agents tab

1. fixed: ten column table with headers, 45px rows.
2. fixed: pencil, stop and kebab (Open and edit, Edit note, Archive, Delete, Copy id). Menu text is
   smaller, see new item 141.
3. fixed: panel header with lead line, seen age and chat icon.
4. fixed: status dot (dotSize lg with halo) and tag in the first column; model tag dot through
   Tag dot; stop, branch and undo icons use IconButton tone danger, info and accent; item ref is
   neutral as Node. The table is now Tessera TrackTable: onRowClick opens the drawer, rowDetail
   draws the full message and the note editor, and no row is tinted (Node only draws a stale
   chip and a warning dot). Columns take the Node widths through the TrackColumn `width` of
   Tessera 0.85 (status 6.8rem, agent 9.5rem, model 4.4rem, kind 4rem, item 3.6rem, branch 7rem,
   time 4.2rem, now 10rem, actions 8rem; the message and note keep fr 2 and 1), measured equal to
   Node at 1440, 1300, 1200, 1000, 800 and 700. The status column is a dot of 22px at 420 and below
   40rem, as Node. Node drops branch at 80rem, kind and now at 70rem, item at 56rem, model at 48rem and
   the rest at 40rem. Since Tessera 0.87 the columns take `hideBelow` at Node's steps (branch 80rem,
   kind and now 70rem, item 56rem, model 48rem, time and last message 40rem) and a hidden column
   leaves no gap; the width clamp and its two model files are gone. Only the status dot at 420 (24px, was 22px) and
   below 40rem still comes from one container rule that sets the status width.
5. fixed: sonnet green, haiku blue, opus grey.
6. fixed: Show finished disclosure with the hint; counts now equal (Brock 233, RotP 93).
7. fixed: search field removed.
8. fixed: time cell "37m" over "49 · 54".
9. fixed: headers kept and the empty sentence centred.

### Board: Kanban tab

1. fixed: the Threads, status and subject selects are content wide (Select `width="fit"`) and the
   search is 220px (SearchInput `width="narrow"`); FieldFit and TestsFit are gone. At 420 Node
   stretches the search to the full row; the toolbar Card takes `container` (Tessera 0.87) so the
   `narrow-fill` search fills the row at 420. The selected Group by and Order segments are still the palette outline (new item 124).
2. fixed: three columns share the full width (they are panels as tall as the tallest, new item 130).
3. fixed: vertical dots kebab, and no drag handle on the cards (KanbanBoard `grip={false}`).
4. fixed: the needs you card has the red left edge (KanbanBoard cardEdgeTone).
5. fixed: inline add form with title, thread select set to the column thread, Add and Cancel.
6. fixed: card menu has the header, Edit, inline Move to status and Reassign to thread selects
   (DropdownMenu item `kind="select"`), then Archive, Delete and Copy id; the actions close the
   menu, a select pick keeps it open (the card moves and the menu goes with it). The row text size is
   item 141. Checked again in the close-out with a headless shot: the label sits above its select in the
   Argus menu, as in Node (`docs/visual-audit/x-card-kebab.png` shows the Node menu).
7. fixed: no collapse chevron on the column headers (KanbanBoard `foldable={false}`).
8. fixed: "none" in an empty column and "6 open / 41" counter.
9. fixed: Threads filter opens a checklist (a little wider than Node).

### Board: Tests tab

1. fixed: same toolbar panel as Kanban and Hide passed as a link.
2. fixed: one line test rows with tone outcome buttons (labels capitalised, see new item 135).
3. fixed: bundle head on one row with tags, progress bar and Unhand.
4. fixed: full width launch field with Copy and Play, run line, "run in" and "from launcher" chip.
   The Copy button is green outlined and the launcher chip accent coloured, see new item 125.
5. fixed: Terminal box with Full log.
6. fixed: done rows with outcome tag, "you · age", lead notified tag, amber left bar.
7. fixed: chips show the thread id.
8. fixed: Show closed (13) as a link.

### Board: Builds tab

1. fixed: command row with Copy, Play, run status and Log.
2. fixed: bundle chip with link icon and status tag.
3. fixed: set by lead right aligned (age rounding differs, see Page shell item 5).
4. fixed: current launcher has the green edge.
5. fixed: the panel header is one line, "Builds 9 to start · 1 retired What you can start...", with
   the divider under it (the shell Panel).
6. fixed: Terminal box with Full log.
7. fixed: master tag neutral.

### Board: Worktrees tab

1. fixed: the table fits its box, Request and Actions visible (Node still clips them).
2. fixed: title and summary on one row, the explainer and the segmented control at the right.
3. fixed: lowercase status tags with Node tones.
4. fixed: branch as plain mono text.
5. fixed: the Touched column uses the shared age ("25h ago"); the leftover rows read "last changed 22h
   ago" on one line, as Node (item 152). No place on the Worktrees tab uses another age text now.
6. fixed: no table footer.
7. fixed: Copy label after the path. The path column is wide enough for the full path, see new item 134.

### Drawers: what every drawer shares

1. fixed: 768px at 1440 (Node 760px), full width at 420.
2. fixed: title is white sentence case (RecordDrawer uses Drawer look record) and RecordDrawer passes
   Drawer `stackNarrow` (Tessera 0.85): at 420 the title takes the full width and the close X sits in
   the tools row, as Node. RecordDrawer passes `headerActionsRow="eyebrow"` (0.87), so the tools sit
   in the row of the id and the status tag as Node.
3. fixed: archive and trash are icon buttons in the header.

### Drawer: work item

1. fixed: Status and Thread selects are content wide and the Subject sits on the same row.
2. fixed: uppercase titles with the middle dot, empty lines centred.
3. fixed: Add buttons say Add.
4. fixed: question card with tags, kebab, context and an indented reply.

### Drawer: test

1. fixed: Mode, Item, Covers and Build controls.
2. fixed: two Expected cards, PROVES IT WORKS and PROVES IT IS BROKEN.
3. fixed: steps are a Stepper (vertical, size sm, ring dots). The labels are plain text, so
   inline marks in a step show without their formatting; Node draws a rule between steps where
   Stepper draws a connector and spaces the steps wider.
4. fixed: Result card with "Not run yet." centred.
5. fixed: run buttons toned.
6. fixed: tools are in the header like every drawer.

### Drawer: agent

1. fixed: WHERE section with Model, Kind, Branch and Worktree.
2. fixed: Item select and Thread.
3. fixed: Task, Note, Result, Where, Timeline with seconds and age.
4. fixed: title is white regular (RecordDrawer look record); the id line carries the agent id label as Node.

### Drawer: thread

1. fixed: open work rows are one line, nowrap id.
2. fixed: title is white regular; tools are all there (copy, previous, next, Expand, chat,
   copy, kebab).
3. fixed: the subject is an InlineEdit with look link, and the T-6 and T-7 links are neutral
   underlined (new item 126).

### Menus, popovers and editors that only open

1. fixed: More menu lists Trash, Resources and locks, Legend, Show hidden, Purge (counts in
   parentheses, Node right aligns them).
2. fixed: Trash view is a Trash panel with the count and the Back button, no tab selected. Node swaps the tab body for a "Trash 0 in trash [Back to the board]" panel;
   Argus shows a green outlined "Back to the board" button and plain text, with the Kanban tab still
   selected.
3. fixed: Send down on a card opens the same anchored popover as Send to all.
4. fixed: see the Agents and Kanban sections.
5. fixed: project switcher on every board.

### Narrow window (420px)

1. fixed: header, tabs, Focus and first rows on the first screen. Kanban and Tests bodies differ at
   420, see new items 136 and 137.
2. fixed: 2 cards per row.
3. fixed: header folds like Node.
4. fixed: kept as is (scroll chevron on the tab bar is better).

### Empty, busy and not found states

1. fixed: empty tests: same sentence in a compact panel.
2. fixed: empty agents keeps the table frame and headers.
3. fixed: "none" in empty columns.
4. fixed: unknown board: the SPA shows Not found ("There is no board for ...") once the portfolio
   has loaded and that name is not in it.
5. fixed: Node's source settles it. `setStatus('Connecting to the dashboard...')` runs, then the very
   next line awaits `claude.use('db')`, which resolves at once in `claude-shim.js`, and `setStatus('')`
   clears it before the browser paints; the stream state is never shown while it opens. Nothing is on
   screen except the board header, which says "waiting for the board" until the board document
   arrives. Argus said "connecting" there; it says "waiting for the board" now. The notice for a
   stream that fails is Node's "Dashboard unreachable, retrying." in the danger tone
   (item 144).

### New differences

119. fixed: Processes rows are 40px, the needs-a-look row draws the left bar only (rowState edge)
    and the project cell is Link tone link. (Overview, Processes)
120. fixed: robot and flask icons on the agents and tests tiles (the items tile keeps columns, as Node). Project card stat tiles use other icons: Node robot, book and flask or person; Argus
    gear, columns and checklist. (Overview, Dashboards)
121. fixed (Tessera 0.85): both tab bars take `badgeFill="solid"`, the solid badge with dark text in the Node tones. (All tab bars)
122. fixed: both tab bars use Tabs `size="lg"` (40px, measured 40px as Node, the bar sits at 66px on the overview and 128px on boards) and `selectedTone="neutral"` (the active label is the text colour, not the palette primary). (All tab bars)
123. fixed: the overview chip filters (levels, kinds, projects) are Tessera Tag radios, pill, `size="lg"` (24px) and `selectedTone="neutral"` (filled grey when selected), in a radiogroup. TagPicker 0.84 takes `pill` but passes neither `selectedTone` nor a Tag size, so ChipFilter composes Tag directly. The notification toggles are pill, `size="lg"`, `selectedTone="neutral"`, filled grey when on. (Overview Activity and Notifications)
124. fixed: every SegmentedControl in Argus takes `selectedTone="neutral"` (grey fill, no palette outline): Kanban Group by, Order and Post as, Tests Group by and Order, Activity view, Worktrees filter and the Send down kind. ToggleGroup has no `selectedTone` in 0.84, but Argus uses no ToggleGroup. (Kanban, Tests, Activity, Worktrees, Send down)
125. fixed: Copy is a tertiary (grey) button and the launcher chip link is neutral. Palette leaks on the Tests tab: the Copy button is a green outline (Node grey) and the
    "from launcher" chip is accent coloured (Node neutral). (Tests)
126. fixed: BoardMarkdown passes linkTone neutral, so T-n and I-n links are neutral underlined, and
    since Tessera 0.85 the web links take `linkTone="info"` (blue, as Node) apart from the ids
    (`idLinkTone="neutral"`). (Thread drawer)
127. fixed: decision cards and the waiting-on-you bundle and test rows have the amber left edge
    (Card edge in the warning tone). (Board Activity)
128. fixed: the waiting row shows the title only, no image thumbnails (the unused ProofGallery is
    gone). (Board Activity)
129. fixed: Work item description editor opens one pane with Write and Preview tabs and a check and
    cross icon pair. (Item drawer)
130. fixed: each Kanban column is as tall as its cards (KanbanBoard `columnHeight="fit"`). (Kanban)
131. fixed: Board header: the title row starts 16px lower (52px against 36px) though the tab bar
    lands in the right place. (Board header)
132. fixed: the page uses Tessera PageRoot, so the document scrolls (the stable scrollbar gutter of Tessera still keeps 6px free on the right). Argus scrolled an inner area (ScrollArea), not the document: full page screenshots clip at
    the window height and the scrollbar sits inside the frame; Node scrolls the page. (Every screen)
133. fixed: the board chip and the overview registry chip are pills with `size="lg"` (24px); a fault chip is also `outlined` and keeps the warning icon. The fault chip takes the Tag `reason` (Tessera 0.85): the name is in the text colour and only the reason is red, as Node. (Registered strip)
134. fixed: the path column fits the full path (a longer path still truncates in the middle) and the
    size reads "≥ 37 MB". (Worktrees)
135. fixed: outcome buttons read pass, fail, partial, blocked (the shared outcome list, so the test
    drawer and the note dialog follow; the accessible names keep the capital, "Pass T-1"). (Tests,
    test drawer)
136. fixed: at 420 the Kanban columns stack one under the other (KanbanBoard `stack`). (Kanban narrow)
137. fixed: the bundle Disclosure takes wrapEnd, so at 420 the tags, counts, bar and Unhand drop
    under the bundle name. (Tests narrow)
138. fixed, source found: the sign in Node is not in the data and not in the Node code. The board JSON
    from the Node API holds the ASCII letter x (checked on the stream of every board: no U+00D7 anywhere,
    for example "Pad callouts sit well at 1280x720"), `md.js` and `board.html` have no times step, and
    no CSS sets font-feature-settings or font-variant (only tabular-nums on number cells). The sign is
    a glyph substitution of the font: Node loads Inter from Google Fonts (`opsz` axis, 14 to 32), whose
    `calt` feature turns an x between two digits into the multiplication sign, in every Inter text
    on the page (cards, titles, tags, steps), not only in markdown. Measured headless: on the Node page
    "1280x720" is 75.23px wide, the same as "1280\u00d7720", and 73.38px with `calt` off. Tessera ships
    its own Inter file (InterVariable) whose `calt` has the arrows but not this rule: on the Argus page
    "1280x720" is 73.38px with `calt` on or off. So Argus draws the sign itself in text it owns:
    `smartTimes` in `format/` (same rule as Tessera Markdown `smartTimes`: between digits, never in a
    hex value such as 0x1F) is applied to the plain text of cards and rows (`plainText`, so the test
    drawer steps, agent lines, notes), the item, test and thread titles, the item drawer test cards
    and the feed test line; markdown keeps `smartTimes` of Tessera. Text Argus passes to a Tessera
    component without a hook (a tag label, a table cell, a title attribute) still shows the letter x,
    see item 160 (needs Tessera). (Tests, drawers)
139. fixed: the text of an item, agent or thread field opens its editor on click; the "Edit"
    links are gone. The agent drawer says "Claude id". (Drawers)
140. fixed: the Send down popover label reads "for Bxel" with the name bold, the Send down button
    sits at the left under the box with "Ctrl+Enter sends" beside it, and the editor has
    preview={false}, so no eye icon. (Send down)
141. fixed: the card, agent, thread, test and More menus take `itemSize="roomy"`. (Menus)

142. fixed: the Worktrees subject, branch and path cells are Tessera FoldText (`fill` since 0.85, so a
    long text no longer widens the table), the path with `truncate="middle"`, and the one-off CellText
    and its CSS are gone. The path takes FoldText `hug` (0.87), so the Copy button sits right after the text as
    Node. Still open: the cut is by characters, so the end of the file name is not always kept.
    (Worktrees)
143. fixed: the Builds command line and the Tests launch command are a Tessera TextInput `readOnly`
    `mono` in a ButtonRow `lead` beside Copy, Play and Log (0.85). The field is 12px mono, 28px high
    and 1249px wide at 1440 against Node 12px, 28px and 1250px; LauncherCommand.css and the Tests
    command CSS are deleted. A command longer than the field scrolls inside it, where Node wraps.
    (Builds, Tests)

144. fixed: the notice for a lost stream reads "Dashboard unreachable, retrying." in the danger tone,
    as Node (`.notice.bad`); it said "The dashboard is unreachable. Retrying." in the warning tone.
    The sign-in sentence is Node's: `The dashboard asks you to sign in again: run "dashboard open".`
    (Every screen)
145. fixed: the board header says "waiting for the board" until the board document arrives, as Node
    (it said "connecting"; see Empty states item 5). (Board header)
146. fixed: the registered chip tooltips ("Board closed since 5m:", "a wake was sent 5m ago and has
    not been answered yet"), the retired chip ("retired 3d ago by ..."), the leftover delete text, the
    worktree request cell ("asked to delete · waiting" and "5m ago", answered "lead 5m ago") and the
    results history ("2h ago · you") use the shared Age instead of the Tessera relative text.
    (Tooltips, Builds, Worktrees, Tests drawer)
147. fixed: "Show done (35)" is a toggle button that reads "Hide done (35)" when on, with Node's tooltip,
    as the Tests tab "Show passed"; it was a checkbox with a fixed label. (Kanban)
148. fixed: Processes: the "All" and "Needs a look" pills and the rule chips take size lg and the grey
    selected fill (the pills had the white outline); the number cells are 13px dim, the name and pid
    13px, as Node (they were 12px bright and 14px). (Overview Processes)
149. fixed: item drawer question and decision cards: the head reads "Q-29 question · needs a decision
    · lead · 24h answered" and a reply "you · 23h", as Node (it read "lead, 24h ago");
    the card has the kebab (Archive, Delete, Copy id); the question and reply text is 14px and the
    context text dim (it was 12px bright). The Activity list in the drawer shows the short age "22h"
    without "ago", and the Trash cell reads "3m · lead" with the long text as the tooltip.
    (Item drawer, Trash)
150. fixed: the tests of an item show the build chip, "bundle-15 · handed to you" when it is the handed
    build; the fill of that chip differs, see item 157. (Item drawer)
151. fixed: the item drawer eyebrow has a gap between the id and the tags (they touched). (Item drawer)
152. fixed: Worktrees Touched and leftover cells read on one line ("last changed 22h ago"); the request
    chip reads "asked to check · waiting" with the age after it and no "asked" word. (Worktrees)
153. fixed: the Copy button of the launch command (Tests) has no icon and is the same Button as the
    Builds one; "Unhand" is the neutral outline, not the palette green. (Tests, Builds)
154. fixed: the registered chips (board and overview) draw the project name bold in the text colour with
    a green or amber dot, the faulted one with a red icon and the reason in red. The name took the
    status colour before. (Registered strip)
155. fixed: the clock icon in the overview summary has the Node gap before "updated 11m ago".
    (Overview Dashboards)

161. fixed: the Argus mark is Tessera's `BrandMark app="argus"` in `currentColor` (0.87), white on the
    dark app through the text colour; Argus's own Logo component and its SVG parser are gone. It
    sits in the overview header (`md`, before "Argus") and the board header (`lg`). The BrandMark is
    drawn in a square box, so its steps are a little wider than the old wide Logo (24 and 48px, were 26 and
    39px). `apps/web/assets/argus-logo.svg` is the old letter mark, no longer used by the tray script. (Headers)
162. fixed: the favicon stays `apps/web/public/static/favicon.svg`, which already switches black on
    light chrome and white on dark with one `prefers-color-scheme` rule. Tessera 0.87 ships no
    single adaptive favicon file (`brand/argus.svg` is black, `brand/dark-ground/argus.svg` white), so
    there is nothing to switch to. (Shell)
163. fixed: the Kanban toolbar Card takes `container`, so the `narrow-fill` search fills the row at
    420. (Kanban)
164. fixed: the Agents table uses `hideBelow` at Node's steps; see Agents 4. (Agents)
165. fixed: the Worktrees path cell hugs its text and every record drawer puts its tools in the id row;
    see Worktrees 142 and Drawers 2. (Worktrees, Drawers)

### Still open after the close-out

Closed with Tessera 0.96 (spec now ^0.96.0), items 156 to 160, 142 and the card name:

- 156: fixed: bundle names (the Tests bundle heads, the closed bundle heads and the group heads)
  are Text `font="title"` (Chakra Petch 600) at the body size, 14px, as Node. They were mono 14px.
- 157: fixed: Copy (Builds and Tests launch command) and Unhand are Button `variant="quiet"`.
  "handed to you" is a Tag `fill="solid"` (neutral grey) in the bundle tags, the test facts, the
  item drawer build chip and the feed test line. "ready" (Tests and Builds), the lead "active" and
  "paused" chips and the worktree hygiene chips are Tag `fill="tint"`. The kept chip is a neutral
  tinted Tag with `edge="dashed"` (Tessera Tag edge), the dashed grey chip of Node.
- 158: fixed: "needs you", "conflict" and "archived" (Kanban and Tests filter rows) are Checkbox
  `boxed`. Group by and Order are SegmentedControl `size="sm-plus"` (28px segments, 13px), Kanban
  and Tests.
- 159: fixed: the Processes table takes `headerRules={false}` (no vertical header rules) and
  `rowTone` (warning, an amber left bar on needs-a-look rows); `rowState` and its helper are gone.
- 160: fixed: Text and FoldText carry `smartTimes` where free text reaches them: item, test
  and bundle titles in Text, the Worktrees subject in FoldText. Branch names and paths keep the
  letter x on purpose. Still open: text that is not a Tessera child (a `title` attribute, the
  InlineEdit display) keeps the `format/` helper.
- 142: fixed: the Worktrees path cell is FoldText `truncate="path"` (first folder and the file
  name kept, the middle cut).
- Overview card name: fixed: Text `variant="title" size="lead"`, 15px as Node.

Other (decision for the lead):

- The Agents tab shows a "The reconciler flagged N records" callout (`ReconcilePanel`) that the Node
  page does not have. Node never renders the reconciler report in the board page (`/api/agents/reconcile`
  is read by the CLI only). It is the one Argus-only block left on a board; it is kept, since removing
  a built feature is not a parity call to make alone.
- Small size gaps, fixed: the process rule chips (name, count and memory) and the Processes rule
  column are now 13px (Text `size="md"`), as Node. The master label on the overview card is bold
  mono (600) as Node and now takes the dim tone. Activity one line event rows are 34px as Node (row
  padding is the 6px size token, was the 4px space step).
- Live run elapsed text, fixed: an Agents run row now shows the Node text ("42s", "1m 05s", "1h 02m")
  and ticks every second while a run is live. It uses the shared `useNow` hook with a one second
  tick (`SECOND_TICK_MS`) and the same `runSpan` formatter as a finished run.

### Data differences, re-checked

Finished counts now equal (RotP 93, Brock 233). I-86 shows "Tests 0/0 passing" as Node. Bundle chips
read "closed" and "ready" as Node. Notification ages match. Processes totals match (Argus lists the
same process rows from the Node API through the proxy). The registered strips show the same wake
faults as Node. Bxel palette primary is now #e0245e (owner approved palette v3), not a parity gap.

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
   project chip, "/ lead", and "toast shown" muted under it. Argus: an 83px table row with
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
   lead I-88 7m" and a kebab, the context, the question in bold, the option buttons (the
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
   test to run · title", "lead 7h" and an Open button. Argus: a brown header band BUNDLE TO
   TEST, a meta line, then the title and an "Open in Tests" link. Fix: `TestBody.tsx` as one
   ListItemRow.
5. Work log. Node: short ages ("26m"), the source in link blue (leads, portfolio, lead), the
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
   "ready" green, "handed to you" grey filled, "lead", "3 pass · 2 partial", a green and
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
   ago" and a blue "lead notified" tag at the right, the note on one truncated line. Argus:
   "Partial lead / 40 minutes ago", the note in full, and a "Run again" text button on every
   row (Node keeps it in the kebab). Fix: `DoneList.tsx` rows like the open rows, RerunButton inside
   TestMenu.
7. Thread chip. Node shows the thread id ("lead"); Argus shows the session name ("RotP",
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
3. Head. Node: "set by lead · 3d ago" right aligned. Argus: inline after the tags, "2 days
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
   then "rev 5, updated by lead 8h ago". Argus: a status tag, then the status select and the
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
2. Missing Item select. Node: "Item [I-49 Archipelia B3 ...]" select and "Thread lead".
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
2. Header. Node: eyebrow "lead [active] idle [coord]", title "Brock", tools: copy link,
   previous, next, Expand, chat icon, copy, kebab, close. Argus: "BROCK lead" in accent
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

The 102 side by side images of the re-audit are in `X:/argus/docs/visual-audit/` (Node on the left
or on top, Argus on the right or below; a magenta bar separates them). Name pattern:
`<viewport>-dark-<screen>.png` for whole pages (Argus clips at the window height, see new item 132),
`tab-<board>-<tab>.png` for a tab from the tab bar down at 1440, `x-<what>.png` for menus, popovers
and states.

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
- `X:/argus/docs/visual-audit/x-decision.png`
- `X:/argus/docs/visual-audit/x-item-edit.png`
- `X:/argus/docs/visual-audit/x-loading-board.png`
- `X:/argus/docs/visual-audit/x-loading-overview.png`
- `X:/argus/docs/visual-audit/x-ov-send-all.png`
- `X:/argus/docs/visual-audit/x-ov-send-down.png`
- `X:/argus/docs/visual-audit/x-ov-switcher.png`
- `X:/argus/docs/visual-audit/x-process-rules.png`
- `X:/argus/docs/visual-audit/x-threads-filter.png`
- `X:/argus/docs/visual-audit/x-unknown-board.png`
