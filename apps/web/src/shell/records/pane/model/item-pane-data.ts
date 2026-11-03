/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../data/kanban';
import type { KanbanData } from '../../../../screens/board/tabs/kanban/kanban.type';
import { itemTests, needsYou } from '../../../../screens/board/tabs/kanban/model';
import type { PaneData, RecordData } from '../pane.type';

type Shared = Pick<RecordData, 'boardKey' | 'record' | 'now' | 'ops' | 'acts' | 'feed' | 'open'>;

const itemPaneData = (item: ItemRef, kanban: KanbanData, shared: Shared): PaneData => {
  const allTests = itemTests(kanban.tests, kanban.closedBuilds, true);
  const blocking = kanban.asks.get(item.id)?.find((link) => link.via === 'blocker') ?? null;
  return {
    ...shared,
    item,
    items: kanban.items,
    tests: allTests.filter((test) => test.item === item.id),
    allTests,
    threads: kanban.threads,
    waiting: needsYou(kanban.needs.get(item.id)),
    blocking,
    open: (id) => shared.open(id),
  };
};

export { itemPaneData };
