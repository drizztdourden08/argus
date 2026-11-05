/* @layer renderer-app @kind test */
import type { BoardAction, Entry, WorkItem } from '../../src/data/types';
import { buildWork } from '../../src/screens/board/feed-model';
import type { ManualTest, WorkSources } from '../../src/screens/board/feed-model';

const at = (minute: number): string => `2026-10-26T08:${String(minute).padStart(2, '0')}:00.000Z`;

const entry = (over: Partial<Entry> & { id: string }): Entry => ({
  kind: 'question',
  author: 'lead',
  thread: 'lead',
  status: 'open',
  text: 'text',
  createdAt: at(1),
  options: ['Yes', 'No'],
  ...over,
});

const human = (over: Partial<Entry> & { id: string; kind: string; replyTo: string }): Entry => ({
  author: 'human',
  status: 'new',
  text: '',
  createdAt: at(30),
  ...over,
});

const manual = (over: Partial<ManualTest> & { id: string }): ManualTest => ({
  title: 'a test',
  thread: 'lead',
  mode: 'manual',
  status: 'pending',
  updatedAt: at(5),
  ...over,
});

const step = (over: Partial<BoardAction> & { id: string }): BoardAction => ({
  text: 'do the thing',
  status: 'pending',
  thread: 'lead',
  createdAt: at(2),
  ...over,
});

const work = (over: Partial<WorkSources>) =>
  buildWork({ entries: [], actions: [], tests: [], log: [], items: [], ...over });

const plan = (over: Partial<WorkItem> & { id: string }): WorkItem => ({
  title: 'a plan',
  status: 'open',
  thread: 'lead',
  type: 'test-plan',
  ...over,
});

const run = (over: Partial<WorkItem> & { id: string }): WorkItem => ({
  title: 'a run',
  status: 'open',
  thread: 'lead',
  type: 'test-run',
  stage: 'ready',
  assignee: 'owner',
  updatedAt: at(6),
  fields: { plan: 'I-5' },
  ...over,
});

const ids = (cards: { data: { id: string } }[]): string[] => cards.map((card) => card.data.id);

export { at, entry, human, ids, manual, plan, run, step, work };
