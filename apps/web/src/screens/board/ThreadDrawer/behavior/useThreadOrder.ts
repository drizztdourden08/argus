/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { ProjectThread } from '../../../../data/agents';
import { useCollection } from '../../../../data/react';
import { COORDINATOR_ROLE } from '../ThreadDrawer.constants';

const useThreadOrder = (boardKey: string): string[] => {
  const threads = useCollection<ProjectThread>(boardKey, 'threads');
  return useMemo(
    () =>
      threads.docs
        .filter((thread) => !thread.deletedAt && !thread.mergedInto && !thread.placeholder)
        .sort((a, b) => Number(b.role === COORDINATOR_ROLE) - Number(a.role === COORDINATOR_ROLE) || a.id.localeCompare(b.id))
        .map((thread) => thread.id),
    [threads.docs],
  );
};

export { useThreadOrder };
