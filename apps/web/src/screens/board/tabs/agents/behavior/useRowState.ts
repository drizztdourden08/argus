/* @layer renderer-app @kind logic */
import { useCallback, useMemo, useState } from 'react';
import type { RowState } from '../AgentsTab.type';

const flip = (set: ReadonlySet<string>, id: string): ReadonlySet<string> => {
  const next = new Set(set);
  if (!next.delete(id)) next.add(id);
  return next;
};

const useRowState = (): RowState => {
  const [openMessages, setOpen] = useState<ReadonlySet<string>>(new Set());
  const [editing, setEditing] = useState<string | null>(null);
  const toggleMessage = useCallback((id: string) => setOpen((set) => flip(set, id)), []);
  const stopEdit = useCallback(() => setEditing(null), []);
  return useMemo(() => ({ openMessages, editing, toggleMessage, startEdit: setEditing, stopEdit }), [openMessages, editing, toggleMessage, stopEdit]);
};

export { useRowState };
