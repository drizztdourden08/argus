/* @layer renderer-app @kind test */
import type { PaneData } from '../../src/shell/records/pane/pane.type';
import { NOW, itemRef } from './kanban-docs';

const paneData = (extra: Partial<PaneData> = {}): PaneData => ({
  boardKey: 'alpha',
  item: itemRef('I-1'),
  items: [],
  tests: [],
  allTests: [],
  threads: [],
  agents: [],
  record: { questions: [], decisions: [], notes: [], todos: [], timeline: [] },
  waiting: false,
  blocking: null,
  now: NOW,
  ops: {} as never,
  acts: {} as never,
  feed: {} as never,
  open: () => undefined,
  ...extra,
});

export { paneData };
