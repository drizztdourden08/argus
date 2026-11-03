/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { ActionDoc, Entry, Item, ThreadChoice } from '../../../../../data/kanban';
import { useCollection } from '../../../../../data/react';
import type { Build, Test } from '../../../../../data/types';
import { OP_KINDS, askLinks, needsMap } from '../model';
import { signalsMap } from '../attention';
import type { KanbanData, SoftDoc } from '../kanban.type';
import { useTrails } from './useTrails';

const isLive = (doc: SoftDoc): boolean => !doc.deletedAt && !doc.archivedAt;

const useKanbanData = (boardKey: string): KanbanData => {
  const items = useCollection<Item>(boardKey, 'items');
  const threads = useCollection<ThreadChoice>(boardKey, 'threads');
  const entries = useCollection<Entry>(boardKey, 'entries');
  const actions = useCollection<ActionDoc>(boardKey, 'actions');
  const tests = useCollection<Test>(boardKey, 'tests');
  const builds = useCollection<Build>(boardKey, 'builds');
  const closedBuilds = useMemo(() => new Set(builds.docs.filter((build) => build.closedAt).map((build) => build.id)), [builds.docs]);
  const needs = useMemo(
    () => needsMap({
      entries: entries.docs.filter((entry) => !OP_KINDS.includes(entry.kind) && isLive(entry)),
      actions: actions.docs.filter(isLive),
      tests: tests.docs.filter((test) => isLive(test) && !(test.build && closedBuilds.has(test.build))),
    }),
    [entries.docs, actions.docs, tests.docs, closedBuilds],
  );
  const asks = useMemo(() => askLinks(items.docs, entries.docs), [items.docs, entries.docs]);
  const signals = useMemo(
    () => signalsMap({ tests: tests.docs, closedBuilds }),
    [tests.docs, closedBuilds],
  );
  const trails = useTrails(boardKey, items.docs);
  return {
    ready: items.ready && threads.ready,
    items: items.docs,
    threads: threads.docs,
    entries: entries.docs,
    actions: actions.docs,
    tests: tests.docs,
    closedBuilds,
    needs,
    asks,
    signals,
    trails,
  };
};

export { useKanbanData };
