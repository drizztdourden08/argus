/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useCollection, useDoc } from '../../../../../data/react';
import type { Worktree, WorktreesMeta } from '../../../../../data/worktrees';
import { compareRows } from '../model';
import type { WorktreesData } from '../worktrees.type';

const useWorktreesData = (boardKey: string): WorktreesData => {
  const docs = useCollection<Worktree>(boardKey, 'worktrees');
  const meta = useDoc<WorktreesMeta>(boardKey, 'meta', 'worktrees');
  const rows = useMemo(() => [...docs.docs].sort(compareRows), [docs.docs]);
  return { ready: docs.ready && meta.ready, meta: meta.doc, rows };
};

export { useWorktreesData };
