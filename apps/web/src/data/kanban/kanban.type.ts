/* @layer renderer-app @kind types */
import type { PostResult } from '../api';
import type { RequestedBy, WaitsOn } from '../deps';
import type { ItemTyping } from '../types/item.type';

interface Criterion {
  n?: number;
  text: string;
  done: boolean;
}

interface Item extends ItemTyping {
  title: string;
  description?: string | null;
  subject?: string | null;
  blocker?: string | null;
  thread: string;
  status: string;
  rank?: number;
  rev?: number;
  ac?: Criterion[];
  createdAt?: string;
  createdBy?: string | null;
  updatedAt?: string;
  updatedBy?: string | null;
  archivedAt?: string | null;
  deletedAt?: string | null;
  waitsOn?: WaitsOn | WaitsOn[];
  requestedBy?: RequestedBy | RequestedBy[];
}

interface ThreadChoice {
  name?: string;
  session?: string | null;
  role?: string;
  status?: string;
  placeholder?: boolean;
  lastSeen?: string | null;
  mergedInto?: string | null;
  archivedAt?: string | null;
  deletedAt?: string | null;
}

interface OpTarget {
  c: string;
  id?: string;
}

interface OwnerOp {
  kind: string;
  target?: OpTarget;
  set?: Record<string, unknown>;
  to?: Record<string, unknown>;
  do?: string;
  n?: number;
  m?: number;
  text?: string;
  prev?: Record<string, unknown>;
  reason?: string;
  baseRev?: number;
}

interface OpOutcome {
  status?: string;
  outcome?: string | null;
  current?: Record<string, unknown>;
  by?: string;
}

interface Entry extends OwnerOp, OpOutcome {
  author?: string;
  item?: string | null;
  thread?: string | null;
  replyTo?: string | null;
  createdAt?: string;
  appliedAt?: string | null;
  ops?: OwnerOp[];
  results?: OpOutcome[];
  archivedAt?: string | null;
  deletedAt?: string | null;
}

interface ActionDoc {
  text?: string;
  item?: string | null;
  thread?: string | null;
  status?: string;
  rank?: number;
  rev?: number;
  createdAt?: string;
  deletedAt?: string | null;
  archivedAt?: string | null;
}

interface ActionInput {
  text: string;
  item: string;
  thread: string;
}

interface EditRef {
  c: string;
  id: string;
  rev?: number;
}

interface EditChange {
  field: string;
  prev: string;
  value: string;
}

interface SoftRequest {
  kind: SoftKind;
  id: string;
  c?: string;
  undoes?: string;
}

interface PurgeRequest {
  criteria: object;
  expect: object;
  confirm: string;
}

interface PlaceAmong {
  before?: string;
  after?: string;
}

interface SentOp {
  id: string;
  result: PostResult;
}

type SoftKind = 'archive' | 'unarchive' | 'delete' | 'restore';

interface CriterionOp {
  do: 'add' | 'set' | 'rm' | 'done' | 'undo' | 'move';
  n?: number;
  text?: string;
  prev?: string;
  to?: number;
}

interface ItemInput {
  title: string;
  thread: string;
  status: string;
  parent?: string;
  assignee?: string;
}

interface SentBulk {
  ids: string[];
  results: PostResult[];
}

interface OpVerdict {
  ok: boolean;
  message: string;
}

type ItemRef = Item & { id: string };

type EditField = 'title' | 'subject' | 'description';

type MoveTarget = PlaceAmong & { status?: string; thread?: string };

export type {
  ActionDoc, ActionInput, CriterionOp, EditChange, EditField, EditRef, Entry, Item, ItemInput, ItemRef, MoveTarget, OpOutcome, OpVerdict, OwnerOp, PlaceAmong,
  PurgeRequest, SentBulk, SentOp, SoftKind, SoftRequest, ThreadChoice,
};
