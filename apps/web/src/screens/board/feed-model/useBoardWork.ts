/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useCollection } from '../../../data/react';
import type { BoardAction, Entry, LogDoc, WorkItem } from '../../../data/types';
import { buildWork } from './build-work';
import type { BoardWork, BuildRef, ManualTest } from './feed.type';

const useBoardWork = (key: string): BoardWork => {
  const entries = useCollection<Entry>(key, 'entries');
  const actions = useCollection<BoardAction>(key, 'actions');
  const tests = useCollection<ManualTest>(key, 'tests');
  const builds = useCollection<BuildRef>(key, 'builds');
  const log = useCollection<LogDoc>(key, 'log');
  const items = useCollection<WorkItem>(key, 'items');
  const ready = [entries, actions, tests, builds, log, items].every((collection) => collection.ready);
  return useMemo(
    () => ({
      ready,
      ...buildWork({ entries: entries.docs, actions: actions.docs, tests: tests.docs, builds: builds.docs, log: log.docs, items: items.docs }),
    }),
    [ready, entries.docs, actions.docs, tests.docs, builds.docs, log.docs, items.docs],
  );
};

export { useBoardWork };
