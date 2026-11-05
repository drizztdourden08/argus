/* @layer renderer-app @kind types */
import type { KanbanColumn, KanbanMove } from '@drizztdourden08/tessera';
import type { DocData } from '../../../../../data/api';
import type { DepHistoryRow, DepState } from '../../../../../data/deps';
import type { Entry, ItemRef, PlaceAmong, ThreadChoice } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import type { OpNotice, Option } from '../../../shared';
import type { AttentionKind, Signals } from '../attention/attention.type';

type ItemGroup = 'status' | 'thread' | 'assignee' | 'parent';

type ItemOrder = 'rank' | 'updated';

type EditableField = 'title' | 'subject' | 'description' | 'brief' | 'criterion' | 'todo';

interface KanbanFilters {
  query: string;
  threads: string[];
  status: string;
  types: string[];
  assignees: string[];
  needs: boolean;
  conflict: boolean;
  archived: boolean;
}

interface KanbanPrefs {
  group: ItemGroup;
  order: ItemOrder;
  showDone: boolean;
}

interface StoredView {
  filters: KanbanFilters;
  prefs: KanbanPrefs;
}

interface RawStored {
  filters: string | null;
  group: string | null;
  showDone: string | null;
}

interface BulkSofts {
  ops: DocData[];
  inverse: DocData[];
}

interface Needs {
  questions: number;
  decisions: number;
  todos: number;
  tests: number;
}

type AskKind = 'question' | 'decision';

interface AskLink {
  id: string;
  kind: AskKind;
  via: 'item' | 'blocker';
}

interface TrailInfo {
  depth: number;
  open: number;
}

interface ChildCount {
  done: number;
  total: number;
}

interface CardData {
  id: string;
  item: ItemRef;
  needsYou: boolean;
  needs: Needs | undefined;
  children: ChildCount;
  lastRun: ItemRef | null;
  trail: TrailInfo | null;
  asks: AskLink[];
  signals: Signals;
  attention: AttentionKind[];
  quietMs: number | null;
  conflicted: boolean;
  group?: string;
}

interface CardFacts {
  needs: Needs | undefined;
  children: ChildCount;
  lastRun: ItemRef | null;
  trail: TrailInfo | null;
  asks: AskLink[];
  signals: Signals;
  conflicted: boolean;
}

interface ViewInput {
  items: ItemRef[];
  threads: WithId<ThreadChoice>[];
  needs: ReadonlyMap<string, Needs>;
  asks: ReadonlyMap<string, AskLink[]>;
  signals: ReadonlyMap<string, Signals>;
  trails: ReadonlyMap<string, TrailInfo>;
  notices: readonly OpNotice[];
  filters: KanbanFilters;
  prefs: KanbanPrefs;
  paused: ReadonlySet<string>;
  focus: string;
  showPaused: boolean;
  now: number;
}

interface AddPreset {
  status?: string;
  thread?: string;
  parent?: string;
  assignee?: string;
}

interface FocusChoices {
  options: Option[];
  pausedCount: number;
}

interface ColumnsInput {
  cards: readonly CardData[];
  group: ItemGroup;
  order: ItemOrder;
  showClosed: boolean;
  seeds: readonly string[];
  parents: ReadonlyMap<string, ItemRef>;
}

interface KanbanView {
  columns: KanbanColumn<CardData>[];
  cards: CardData[];
  open: number;
  total: number;
  quiet: number;
  closed: number;
  filtered: boolean;
  parents: ReadonlyMap<string, ItemRef>;
}

type EntryDocs = readonly WithId<Entry>[];

interface NeedsInput {
  entries: EntryDocs;
  actions: readonly WithId<{ item?: string | null; status?: string }>[];
  tests: readonly WithId<{ item: string | null; mode: string; status: string; archivedAt?: string | null }>[];
}

interface DropInput {
  columns: readonly KanbanColumn<CardData>[];
  move: KanbanMove;
  group: ItemGroup;
  order: ItemOrder;
}

type DropPlan =
  | { kind: 'none' }
  | { kind: 'order-off' }
  | { kind: 'reorder'; place: PlaceAmong }
  | { kind: 'group-off' }
  | { kind: 'status' | 'thread'; to: string; place: PlaceAmong };

interface TestScope {
  archivedAt?: string | null;
  deletedAt?: string | null;
  build?: string | null;
}

interface DepLink {
  side: 'waits' | 'asked';
  dep: string;
  project: string;
  item: string;
  title: string;
  thread: string | null;
  state: DepState;
  since: string | null;
  reason: string | null;
}

interface CardChip {
  id: string;
  text: string;
  title: string;
}

interface CardProgress {
  met: number;
  total: number;
  text: string;
  title: string;
}

type DepView = DepLink & {
  history: DepHistoryRow[];
  cancellable: boolean;
};

export type {
  AddPreset, AskKind, AskLink, BulkSofts, CardChip, CardData, CardProgress, ChildCount, CardFacts, ColumnsInput, DepLink, DepView, DropInput, DropPlan, EditableField, EntryDocs, FocusChoices, ItemGroup, ItemOrder, KanbanFilters, KanbanPrefs, KanbanView, Needs, NeedsInput, RawStored, StoredView, TestScope, TrailInfo, ViewInput,
};
