/* @layer renderer-app @kind test */
import type { Entry, ItemRef, ThreadChoice } from '../../src/data/kanban';
import type { WithId } from '../../src/data/react';
import type { CardData, KanbanFilters, KanbanPrefs, Needs, OpNotice } from '../../src/screens/board/tabs/kanban/model';
import { DEFAULT_FILTERS, buildView } from '../../src/screens/board/tabs/kanban/model';

const MIN = 60_000;
const HOUR = 3_600_000;
const DAY = 86_400_000;

const NOW = Date.parse('2026-10-26T12:00:00.000Z');

const before = (ms: number): string => new Date(NOW - ms).toISOString();

const itemRef = (id: string, extra: Partial<ItemRef> = {}): ItemRef => ({
  id,
  title: `Item ${id}`,
  description: '',
  subject: 'general',
  thread: 'worker',
  status: 'todo',
  rank: 1024,
  rev: 3,
  ac: [],
  createdAt: before(30 * DAY),
  updatedAt: before(MIN),
  ...extra,
});

const threadRef = (id: string, extra: Partial<ThreadChoice> = {}): WithId<ThreadChoice> => ({
  id,
  name: id,
  session: id,
  role: 'thread',
  status: 'active',
  ...extra,
});

const entryRef = (id: string, extra: Partial<Entry> = {}): WithId<Entry> => ({
  id,
  kind: 'question',
  author: 'worker',
  status: 'open',
  item: 'I-1',
  createdAt: before(HOUR),
  ...extra,
});

const filters = (extra: Partial<KanbanFilters> = {}): KanbanFilters => ({ ...DEFAULT_FILTERS, ...extra });

const prefs = (extra: Partial<KanbanPrefs> = {}): KanbanPrefs => ({ group: 'status', order: 'rank', showDone: false, ...extra });

const noNeeds: ReadonlyMap<string, Needs> = new Map();

const cardOf = (item: ItemRef): CardData => ({ id: item.id, item, needsYou: false, decisions: 0, quietMs: null, conflicted: false });

interface Setup {
  filters?: Partial<KanbanFilters>;
  prefs?: Partial<KanbanPrefs>;
  needs?: ReadonlyMap<string, Needs>;
  notices?: OpNotice[];
  threads?: WithId<ThreadChoice>[];
  paused?: string[];
  focus?: string;
  showPaused?: boolean;
}

const viewOf = (items: ItemRef[], setup: Setup = {}) =>
  buildView({
    items,
    threads: setup.threads ?? [threadRef('worker')],
    needs: setup.needs ?? noNeeds,
    notices: setup.notices ?? [],
    filters: filters(setup.filters),
    prefs: prefs(setup.prefs),
    paused: new Set(setup.paused ?? []),
    focus: setup.focus ?? '',
    showPaused: setup.showPaused ?? false,
    now: NOW,
  });

const idsIn = (view: ReturnType<typeof viewOf>, column: string): string[] =>
  (view.columns.find((candidate) => candidate.id === column)?.cards ?? []).map((card) => card.id);

export { DAY, HOUR, MIN, NOW, before, cardOf, entryRef, filters, idsIn, itemRef, noNeeds, prefs, threadRef, viewOf };
