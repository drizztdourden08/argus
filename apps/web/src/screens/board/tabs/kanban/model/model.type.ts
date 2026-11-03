/* @layer renderer-app @kind types */
import type { KanbanColumn, KanbanMove, TagUrgencyColor } from '@drizztdourden08/tessera';
import type { DocData } from '../../../../../data/api';
import type { DepColor, DepHistoryRow, DepState } from '../../../../../data/deps';
import type { Entry, ItemRef, PlaceAmong, ThreadChoice } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import type { OpNotice, Option } from '../../../shared';

type ItemGroup = 'status' | 'subject' | 'thread';

type ItemOrder = 'rank' | 'updated';

type EditableField = 'title' | 'subject' | 'description' | 'brief' | 'criterion' | 'todo';

interface KanbanFilters {
  query: string;
  threads: string[];
  status: string;
  subject: string;
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

interface AskTagSpec {
  id: string;
  label: string;
  color: TagUrgencyColor;
  title: string;
}

interface CardData {
  id: string;
  item: ItemRef;
  needsYou: boolean;
  asks: AskLink[];
  quietMs: number | null;
  conflicted: boolean;
}

interface CardFacts {
  needs: Needs | undefined;
  asks: AskLink[];
  conflicted: boolean;
}

interface ViewInput {
  items: ItemRef[];
  threads: WithId<ThreadChoice>[];
  needs: ReadonlyMap<string, Needs>;
  asks: ReadonlyMap<string, AskLink[]>;
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
  subject?: string;
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
}

interface KanbanView {
  columns: KanbanColumn<CardData>[];
  cards: CardData[];
  open: number;
  total: number;
  quiet: number;
  closed: number;
  filtered: boolean;
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
  | { kind: 'status' | 'thread' | 'subject'; to: string; place: PlaceAmong };

interface TestScope {
  archivedAt?: string | null;
  deletedAt?: string | null;
  build?: string | null;
}

interface DepTagSpec {
  key: string;
  color: DepColor;
  label: string;
  title: string;
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

type DepView = DepLink & {
  history: DepHistoryRow[];
  cancellable: boolean;
};

export type {
  AddPreset, AskKind, AskLink, AskTagSpec, BulkSofts, CardData, CardFacts, ColumnsInput, DepLink, DepTagSpec, DepView, DropInput, DropPlan, EditableField, EntryDocs, FocusChoices, ItemGroup, ItemOrder, KanbanFilters, KanbanPrefs, KanbanView, Needs, NeedsInput, RawStored, StoredView, TestScope, ViewInput,
};
