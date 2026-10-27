/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { ProjectThread } from '../../../../data/agents';
import { useCollection } from '../../../../data/react';
import type { Lock, MoreCounts } from '../BoardMore.type';

const isHidden = (thread: ProjectThread): boolean =>
  thread.status === 'done' || thread.status === 'archived' || Boolean(thread.archivedAt) || Boolean(thread.placeholder);

const useMoreCounts = (boardKey: string): MoreCounts => {
  const locks = useCollection<Lock>(boardKey, 'locks');
  const threads = useCollection<ProjectThread>(boardKey, 'threads');
  return useMemo(
    () => ({
      held: locks.docs.filter((lock) => lock.holder).length,
      hidden: threads.docs.filter((thread) => !thread.mergedInto && thread.role !== 'coordinator' && isHidden(thread)).length,
      locks: locks.docs,
    }),
    [locks.docs, threads.docs],
  );
};

export { useMoreCounts };
