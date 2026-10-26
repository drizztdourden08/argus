/* @layer renderer-app @kind types */
import type { CriterionOp, EditField, Entry, ItemInput, ItemRef, MoveTarget, SentOp, SoftKind, ThreadChoice } from '../../../../data/kanban';
import type { WithId } from '../../../../data/react';
import type { Test } from '../../../../data/types';
import type { ItemGroup, ItemOrder, KanbanFilters, KanbanPrefs, Needs, OpNotice } from './model';

interface KanbanData {
  ready: boolean;
  items: ItemRef[];
  threads: WithId<ThreadChoice>[];
  entries: WithId<Entry>[];
  tests: WithId<Test>[];
  needs: ReadonlyMap<string, Needs>;
}

interface KanbanState {
  filters: KanbanFilters;
  prefs: KanbanPrefs;
  patchFilters: (patch: Partial<KanbanFilters>) => void;
  clearFilters: () => void;
  setGroup: (group: ItemGroup) => void;
  setOrder: (order: ItemOrder) => void;
  setShowDone: (show: boolean) => void;
}

interface ItemOps {
  move: (item: ItemRef, to: MoveTarget, message: string) => Promise<boolean>;
  setField: (item: ItemRef, field: EditField, value: string, message?: string) => Promise<boolean>;
  criterion: (item: ItemRef, op: CriterionOp, message: string) => Promise<boolean>;
  create: (item: ItemInput) => Promise<boolean>;
  soft: (kind: SoftKind, item: ItemRef) => Promise<boolean>;
}

type BulkTarget = { status: string } | { thread: string };

interface BulkOps {
  move: (items: readonly ItemRef[], to: BulkTarget) => void;
  soft: (kind: 'archive' | 'delete', items: readonly ItemRef[]) => void;
}

interface NoticeOps {
  notices: OpNotice[];
  dismiss: (notice: OpNotice) => void;
  resend: (notice: OpNotice) => void;
}

interface Selection {
  has: (id: string) => boolean;
  toggle: (id: string) => void;
  clear: () => void;
}

interface SoftDoc {
  archivedAt?: string | null;
  deletedAt?: string | null;
}

interface InlineEditState {
  draft: string | null;
  problem: string | null;
  start: () => void;
  cancel: () => void;
  change: (draft: string) => void;
  commit: () => void;
}

type Threads = readonly WithId<ThreadChoice>[];

interface AddItemDraft {
  title: string;
  thread: string;
  status: string;
  setTitle: (title: string) => void;
  setThread: (thread: string) => void;
  setStatus: (status: string) => void;
  ready: boolean;
  reset: () => void;
  submit: () => void;
}

type Writer = (send: () => Promise<SentOp>, done: string, undo?: (sentId: string) => void) => Promise<boolean>;

export type { AddItemDraft, BulkOps, BulkTarget, InlineEditState, ItemOps, KanbanData, KanbanState, NoticeOps, Selection, SoftDoc, Threads, Writer };
