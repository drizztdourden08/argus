/* @layer renderer-app @kind types */
import type { KanbanColumn, KanbanMove } from '@drizztdourden08/tessera';
import type { DocData } from '../../../../../data/api';
import type { Entry, ItemRef, OpOutcome, OwnerOp, PlaceAmong, ThreadChoice } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';

type ItemGroup = 'status' | 'subject' | 'thread';

type ItemOrder = 'rank' | 'updated';

type EditableField = 'title' | 'subject' | 'description' | 'criterion' | 'todo';

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

interface OpNotice {
  id: string;
  entryId: string;
  bulk: boolean;
  kind: 'conflict' | 'refused';
  op: OwnerOp;
  result: OpOutcome;
}

interface NoticeRow {
  field: string;
  mine: string;
  theirs: string;
}

interface CardData {
  id: string;
  item: ItemRef;
  needsYou: boolean;
  decisions: number;
  quietMs: number | null;
  conflicted: boolean;
}

interface Option {
  value: string;
  label: string;
}

interface ViewInput {
  items: ItemRef[];
  threads: WithId<ThreadChoice>[];
  needs: ReadonlyMap<string, Needs>;
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

export type {
  AddPreset, BulkSofts, CardData, ColumnsInput, DropInput, DropPlan, EditableField, EntryDocs, FocusChoices, ItemGroup, ItemOrder, KanbanFilters, KanbanPrefs, KanbanView, Needs, NeedsInput, NoticeRow, OpNotice, Option, RawStored, StoredView, ViewInput,
};
