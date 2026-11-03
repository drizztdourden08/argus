/* @layer renderer-app @kind test */
import type { Entry, ItemRef, ThreadChoice } from '../../src/data/kanban';
import type { WithId } from '../../src/data/react';
import type { OpNotice } from '../../src/screens/board/shared';
import type { Signals } from '../../src/screens/board/tabs/kanban/attention';
import type { AskLink, KanbanFilters, KanbanPrefs, Needs, TrailInfo } from '../../src/screens/board/tabs/kanban/model';
import { DEFAULT_FILTERS, buildView } from '../../src/screens/board/tabs/kanban/model';

const MIN = 60_000;
const HOUR = 3_600_000;
const DAY = 86_400_000;

const NOW = Date.parse('2026-10-26T12:00:00.000Z');

const before = (ms: number): string => new Date(NOW - ms).toISOString();

const itemRef = (id: string, extra: Partial<ItemRef> = {}): ItemRef => ({
  id,
  title: `Item number ${id.slice(2)}`,
  description: '',
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

interface Setup {
  filters?: Partial<KanbanFilters>;
  prefs?: Partial<KanbanPrefs>;
  needs?: ReadonlyMap<string, Needs>;
  asks?: ReadonlyMap<string, AskLink[]>;
  signals?: ReadonlyMap<string, Signals>;
  trails?: ReadonlyMap<string, TrailInfo>;
  notices?: OpNotice[];
  threads?: WithId<ThreadChoice>[];
  paused?: string[];
  focus?: string;
  showPaused?: boolean;
}

const listsOf = (setup: Setup) => ({
  threads: setup.threads ?? [threadRef('worker')],
  notices: setup.notices ?? [],
  paused: new Set(setup.paused ?? []),
});

const mapsOf = (setup: Setup) => ({
  needs: setup.needs ?? noNeeds,
  asks: setup.asks ?? new Map(),
  signals: setup.signals ?? new Map(),
  trails: setup.trails ?? new Map(),
});

const viewOf = (items: ItemRef[], setup: Setup = {}) =>
  buildView({
    items,
    ...listsOf(setup),
    ...mapsOf(setup),
    filters: filters(setup.filters),
    prefs: prefs(setup.prefs),
    focus: setup.focus ?? '',
    showPaused: setup.showPaused ?? false,
    now: NOW,
  });

const idsIn = (view: ReturnType<typeof viewOf>, column: string): string[] =>
  (view.columns.find((candidate) => candidate.id === column)?.cards ?? []).map((card) => card.id);

export { DAY, HOUR, NOW, before, entryRef, filters, idsIn, itemRef, noNeeds, prefs, threadRef, viewOf };
