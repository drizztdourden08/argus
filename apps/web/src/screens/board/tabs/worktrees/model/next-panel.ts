/* @layer renderer-app @kind logic */
import type { WorktreesMeta } from '../../../../../data/worktrees';
import type { Panel, PanelKind, WorktreeRow } from './model.type';
import { seenOf } from './seen-of';

const nextPanel = (current: Panel | null, row: WorktreeRow, kind: PanelKind, meta: WorktreesMeta | null): Panel | null => {
  const sameRow = current !== null && current.id === row.id;
  if (sameRow && current.kind === kind) return null;
  const keepText = sameRow && kind !== 'remove' && current.kind !== 'remove';
  return { id: row.id, kind, text: keepText ? current.text : '', branch: false, error: null, seen: seenOf(row, meta) };
};

export { nextPanel };
