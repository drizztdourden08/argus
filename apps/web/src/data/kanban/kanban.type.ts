/* @layer renderer-app @kind types */
import type { PostResult } from '../api';

interface Criterion {
  n?: number;
  text: string;
  done: boolean;
}

interface Item {
  title: string;
  description?: string | null;
  subject?: string | null;
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
}

interface ThreadChoice {
  name?: string;
  session?: string | null;
  role?: string;
  status?: string;
  placeholder?: boolean;
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
  item?: string | null;
  status?: string;
  deletedAt?: string | null;
  archivedAt?: string | null;
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
  subject?: string;
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
  ActionDoc, CriterionOp, EditField, Entry, Item, ItemInput, ItemRef, MoveTarget, OpOutcome, OpVerdict, OwnerOp, PlaceAmong, SentBulk, SentOp, SoftKind, ThreadChoice,
};
