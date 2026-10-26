/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import type { ActionDoc, Entry, Item, ThreadChoice } from '../../../../../data/kanban';
import { useCollection } from '../../../../../data/react';
import type { Build, Test } from '../../../../../data/types';
import { OP_KINDS, needsMap } from '../model';
import type { TrashSource } from '../trash';
import type { KanbanData, SoftDoc } from '../kanban.type';

const isLive = (doc: SoftDoc): boolean => !doc.deletedAt && !doc.archivedAt;

const useKanbanData = (boardKey: string): KanbanData => {
  const items = useCollection<Item>(boardKey, 'items');
  const threads = useCollection<ThreadChoice>(boardKey, 'threads');
  const entries = useCollection<Entry>(boardKey, 'entries');
  const actions = useCollection<ActionDoc>(boardKey, 'actions');
  const tests = useCollection<Test>(boardKey, 'tests');
  const builds = useCollection<Build>(boardKey, 'builds');
  const agents = useCollection<TrashSource>(boardKey, 'agents');
  const needs = useMemo(() => {
    const closed = new Set(builds.docs.filter((build) => build.closedAt).map((build) => build.id));
    return needsMap({
      entries: entries.docs.filter((entry) => !OP_KINDS.includes(entry.kind) && isLive(entry)),
      actions: actions.docs.filter(isLive),
      tests: tests.docs.filter((test) => isLive(test) && !(test.build && closed.has(test.build))),
    });
  }, [entries.docs, actions.docs, tests.docs, builds.docs]);
  return {
    ready: items.ready && threads.ready,
    items: items.docs,
    threads: threads.docs,
    entries: entries.docs,
    actions: actions.docs,
    agents: agents.docs,
    tests: tests.docs,
    needs,
  };
};

export { useKanbanData };
