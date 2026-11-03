/* @layer renderer-app @kind logic */
import type { KanbanData } from '../../../../screens/board/tabs/kanban/kanban.type';
import type { RecordRef } from '../../records.type';
import type { PaneDoc } from '../pane.type';
import { asDoc } from './as-doc';

const docOf = (target: RecordRef, kanban: KanbanData, other: readonly { id: string }[]): PaneDoc | null => {
  const lists: Readonly<Record<string, readonly { id: string }[]>> = {
    items: kanban.items,
    entries: kanban.entries,
    actions: kanban.actions,
    tests: kanban.tests,
  };
  const found = (lists[target.collection] ?? other).find((doc) => doc.id === target.id);
  return found ? asDoc(found) : null;
};

export { docOf };
