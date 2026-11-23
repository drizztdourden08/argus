/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useCollection } from '../../../data/react';
import type { BoardAction, Entry, WorkItem } from '../../../data/types';
import { buildWork } from './build-work';
import type { BoardWork, ManualTest } from './feed.type';

const useBoardWork = (key: string): BoardWork => {
  const entries = useCollection<Entry>(key, 'entries');
  const actions = useCollection<BoardAction>(key, 'actions');
  const tests = useCollection<ManualTest>(key, 'tests');
  const items = useCollection<WorkItem>(key, 'items');
  const ready = [entries, actions, tests, items].every((collection) => collection.ready);
  return useMemo(
    () => ({
      ready,
      ...buildWork({ entries: entries.docs, actions: actions.docs, tests: tests.docs, items: items.docs }),
    }),
    [ready, entries.docs, actions.docs, tests.docs, items.docs],
  );
};

export { useBoardWork };
