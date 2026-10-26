/* @layer renderer-app @kind logic */
import { useCallback, useState } from 'react';
import type { PurgeRequest } from '../../tabs/kanban';
import type { BoardMoreState } from '../BoardMore.type';

const useBoardMore = (): BoardMoreState => {
  const [trashOpen, setTrashOpen] = useState(false);
  const [purge, setPurge] = useState<PurgeRequest | null>(null);
  const openTrash = useCallback(() => setTrashOpen(true), []);
  const closeTrash = useCallback(() => setTrashOpen(false), []);
  const closePurge = useCallback(() => setPurge(null), []);
  return { trashOpen, purge, openTrash, closeTrash, openPurge: setPurge, closePurge };
};

export { useBoardMore };
