/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { ProjectThread } from '../../../../data/agents';
import type { Entry, Item } from '../../../../data/kanban';
import { useCollection } from '../../../../data/react';
import { pausedThreads } from '../../tabs/kanban';
import { NO_THREAD } from '../ThreadDrawer.constants';
import { openWork } from './open-work';
import { threadCounts } from './thread-counts';
import type { ThreadView } from '../ThreadDrawer.type';

const useThreadView = (boardKey: string, id: string | null): ThreadView => {
  const threads = useCollection<ProjectThread>(boardKey, 'threads');
  const items = useCollection<Item>(boardKey, 'items');
  const entries = useCollection<Entry>(boardKey, 'entries');
  return useMemo(() => {
    const thread = id ? (threads.docs.find((doc) => doc.id === id && !doc.deletedAt && !doc.mergedInto) ?? null) : null;
    if (!thread) return NO_THREAD;
    return {
      thread,
      work: openWork(items.docs, thread.id),
      counts: threadCounts(entries.docs, thread.id),
      paused: pausedThreads([thread], entries.docs).has(thread.id),
    };
  }, [id, threads.docs, items.docs, entries.docs]);
};

export { useThreadView };
