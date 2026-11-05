/* @layer renderer-app @kind types */
import type { ReorderDrag, SortDirection } from '@drizztdourden08/tessera';
import type { KeyboardEvent, RefObject } from 'react';
import type { ActionDoc, CriterionOp, EditChange, EditField, EditRef, Entry, FieldValue, ItemInput, ItemRef, MoveTarget, SoftKind, ThreadChoice } from '../../../../data/kanban';
import type { WithId } from '../../../../data/react';
import type { Test } from '../../../../data/types';
import type { NoteKind } from '../../owner-posts';
import type { OpNotice, Option } from '../../shared';
import type { Signals } from './attention';
import type { TrashPanel } from './trash-props.type';
import type { AskLink, ItemGroup, ItemOrder, KanbanFilters, KanbanPrefs, KanbanView, Needs, TrailInfo } from './model';
import type { PurgeNotice, TrashRow, TrashSource } from './trash';

interface AgentDoc extends TrashSource {
  item?: string | null;
  status?: string;
}

interface KanbanData {
  ready: boolean;
  items: ItemRef[];
  threads: WithId<ThreadChoice>[];
  entries: WithId<Entry>[];
  actions: WithId<ActionDoc>[];
  tests: WithId<Test>[];
  agents: WithId<AgentDoc>[];
  closedBuilds: ReadonlySet<string>;
  needs: ReadonlyMap<string, Needs>;
  asks: ReadonlyMap<string, AskLink[]>;
  signals: ReadonlyMap<string, Signals>;
  trails: ReadonlyMap<string, TrailInfo>;
}

interface KanbanState {
  filters: KanbanFilters;
  prefs: KanbanPrefs;
  patchFilters: (patch: Partial<KanbanFilters>) => void;
  clearFilters: () => void;
  setGroup: (group: ItemGroup) => void;
  setOrder: (order: ItemOrder) => void;
  setDirection: (direction: SortDirection) => void;
  setShowDone: (show: boolean) => void;
}

interface FocusState {
  focus: string;
  showPaused: boolean;
  setFocus: (thread: string) => void;
  setShowPaused: (show: boolean) => void;
}

interface CriteriaReorder {
  listRef: RefObject<HTMLElement | null>;
  drag: ReorderDrag;
  keys: string[];
  message: string;
  step: (event: KeyboardEvent<HTMLElement>, index: number) => void;
}

interface Adding {
  column: string;
  group: ItemGroup;
}

interface AddPanel {
  column: string | null;
  toggle: (column: string) => void;
  close: () => void;
}

interface Panels {
  add: AddPanel;
  trash: TrashPanel;
}

interface KanbanViewInput {
  data: KanbanData;
  state: KanbanState;
  focus: FocusState;
  notices: NoticeOps;
  now: number;
}

interface KanbanViewResult {
  view: KanbanView;
}

interface RecordActions {
  isBusy: (key: string) => boolean;
  done: (todo: WithId<ActionDoc>) => Promise<boolean>;
  compose: (item: ItemRef, kind: NoteKind, text: string) => Promise<boolean>;
}

interface ItemOps {
  move: (item: ItemRef, to: MoveTarget, message: string) => Promise<boolean>;
  setField: (item: ItemRef, field: EditField, value: FieldValue, message?: string) => Promise<boolean>;
  editRecord: (ref: EditRef, change: EditChange, message: string) => Promise<boolean>;
  criterion: (item: ItemRef, op: CriterionOp, message: string) => Promise<boolean>;
  create: (item: ItemInput) => Promise<boolean>;
  addTodo: (item: ItemRef, text: string) => Promise<boolean>;
  soft: (kind: SoftKind, item: ItemRef) => Promise<boolean>;
}

type BulkTarget = { status: string } | { thread: string };

interface BulkOps {
  move: (items: readonly ItemRef[], to: BulkTarget) => void;
  soft: (kind: 'archive' | 'delete', items: readonly ItemRef[]) => void;
  restore: (rows: readonly TrashRow[]) => void;
}

interface NoticeOps {
  notices: OpNotice[];
  purges: PurgeNotice[];
  dismiss: (notice: OpNotice) => void;
  dismissPurge: (notice: PurgeNotice) => void;
  resend: (notice: OpNotice) => void;
}

interface Selection {
  active: boolean;
  setActive: (active: boolean) => void;
  has: (id: string) => boolean;
  toggle: (id: string, order: readonly string[]) => void;
  mark: (shift: boolean) => void;
  clear: () => void;
}

interface SoftDoc {
  archivedAt?: string | null;
  deletedAt?: string | null;
}

type Threads = readonly WithId<ThreadChoice>[];

interface AddItemDraft {
  thread: string;
  setThread: (thread: string) => void;
  parent: string;
  setParent: (parent: string) => void;
  parentOptions: Option[];
  needsParent: boolean;
  ready: boolean;
  itemOf: (title: string) => ItemInput;
}

interface CancelDep {
  busy: boolean;
  cancel: () => Promise<void>;
}

export type { AddItemDraft, AgentDoc, AddPanel, Adding, BulkOps, BulkTarget, CancelDep, CriteriaReorder, FocusState, ItemOps, KanbanData, KanbanState, KanbanViewInput, KanbanViewResult, NoticeOps, Panels, RecordActions, Selection, SoftDoc, Threads };
