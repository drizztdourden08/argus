/* @layer renderer-app @kind logic */
import { useMemo, useState } from 'react';
import type { KanbanDialog, KanbanData, KanbanState, FocusState, BulkOps, Panels } from '../kanban.type';
import { addPreset } from '../model';
import { trashRows } from '../trash';
import { useSelection } from './useSelection';

const usePanels = (data: KanbanData, state: KanbanState, focus: FocusState, bulk: BulkOps): Panels => {
  const [dialog, setDialog] = useState<KanbanDialog>(null);
  const [trashOpen, setTrashOpen] = useState(false);
  const selection = useSelection();
  const rows = useMemo(() => trashRows(data), [data]);
  return {
    dialog,
    close: () => setDialog(null),
    adding: (column) => setDialog({ kind: 'add', preset: addPreset(state.prefs.group, column, focus.focus) }),
    purging: (scope) => setDialog({ kind: 'purge', scope }),
    trash: { open: trashOpen, count: rows.length, rows, selection, bulk, onToggle: () => setTrashOpen(!trashOpen) },
  };
};

export { usePanels };
