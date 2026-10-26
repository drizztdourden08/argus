/* @layer renderer-app @kind logic */
import { useMemo, useState } from 'react';
import type { AddPanel, Adding, BulkOps, KanbanData, KanbanState, Panels } from '../kanban.type';
import { trashRows } from '../trash';
import { useSelection } from './useSelection';

const usePanels = (data: KanbanData, state: KanbanState, bulk: BulkOps, trash: { open: boolean; onBack: () => void }): Panels => {
  const [adding, setAdding] = useState<Adding | null>(null);
  const selection = useSelection();
  const rows = useMemo(() => trashRows(data), [data]);
  const { group } = state.prefs;
  const column = adding?.group === group ? adding.column : null;
  const add: AddPanel = {
    column,
    toggle: (target) => setAdding(column === target ? null : { column: target, group }),
    close: () => setAdding(null),
  };
  return { add, trash: { ...trash, rows, selection, bulk } };
};

export { usePanels };
