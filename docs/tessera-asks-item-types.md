<!-- @layer docs @kind doc -->
# Tessera asks for the Argus item-type rework (Tessera I-71)

Argus is reworking its board items: typed items (epic, story, task, bug, tech debt, test case, test plan), an assignee on every item, and a compact card plus a tabbed detail pane. These are the pieces Argus needs from Tessera. Until a piece lands, Argus keeps today's look for that part and builds nothing locally.

## Avatar

- **Screen.** The item card header (right side, before the menu), the Overview tab of the item pane, the owner's "Assigned to you" list.
- **Props.** `label` (the person's name, used for the tooltip and screen readers), `initials` (one or two letters) or `icon` (a Tessera icon name, for example `user` for the owner), `color` (a category colour), `size` (`xs` 16 px on the card, `sm` 24 px in the pane), `variant` (`default`, `owner`: a ring around it so the owner's own items stand out).
- **States.** Default; unassigned (a dashed outline circle, no initials); hover shows the tooltip.

## TypeMark

- **Screen.** The left of the card header, the pane title, the Tree view rows, legend rows.
- **Props.** `icon` (Tessera icon name: zap, bookmark, circle-check, bug, wrench, scroll-text, flask), `color` (category colour), `size` (`xs` 16 px, `sm` 20 px), `label` (the type name, for screen readers and the tooltip).
- **States.** Default only; it is a tinted tile with the icon in the colour.

## SignalStrip

- **Screen.** One row on the item card under the title: the attention signals (questions, failing tests, deps, blocked, quiet, agent, trail depth).
- Its props are `items`, a list of `{ id, icon, count?, label, tone, onClick? }` (tone: danger, warning, info, success, neutral). `max?` (otherwise it fits the width). It never wraps: what does not fit folds into a `+N` chip whose tooltip lists the hidden ones.
- **States.** Chips clickable (Tag onClick) or static; the `+N` chip; empty (renders nothing).

## OutcomeStrip

- **Screen.** The test plan runner in the item pane (one cell per test case) and the test plan card (small).
- Its props are `steps`, each `{ id, outcome, label }`, where outcome is pass, fail, partial, blocked, skip, pending or current; `onSelect(id)`; `size` (`xs` on the card, `sm` in the runner).
- **States.** Each outcome toned; `current` outlined; hover shows the label.

## MarkdownEditor: image paste and drop, text counter

- **Screen.** The description field of every item in the pane, and the add-item form.
- **Props.** `onImageFiles(files) => Promise<string>` returns the Markdown to insert (for example `![shot](img:2)`); an image button in the toolbar that opens a file picker and calls the same function; `count(text) => number` so the counter counts text without markup (Argus counts at most 500 characters of text, images excluded).
- **States.** Uploading (a placeholder where the image goes), upload refused (the `error` prop shows the message), over the limit (counter in danger).

## MediaGrid

- **Screen.** The Media tab of the item pane and the board-wide Media tab.
- Its props are `items`, each `{ src, caption, source }` (source is a small tag such as `I-12 description`), `onOpen(index)` opens `MediaViewer` at that image with zoom; `columns?`.
- **States.** Empty (an empty state slot), loading thumbnails, a broken image.

## Text line clamp

- **Screen.** Card titles (3 lines) and dependency trail node titles (1 line).
- **Props.** `clamp={n}`; the full text goes in a tooltip when it is clamped.
- **States.** Clamped with tooltip; not clamped (no tooltip).
