/* @layer renderer-app @kind types */
import type { RequestedBy, WaitsOn } from '../deps';

interface Criterion {
  n: number;
  text: string;
  done: boolean;
}

type BlockedKind = 'item' | 'test' | 'todo' | 'ask' | 'decision' | 'dep-waits' | 'dep-blocked' | 'legacy';

interface BlockedCause {
  kind: BlockedKind;
  id: string;
  label: string;
  who?: string | null;
}

interface ItemImage {
  n: number;
  url: string;
  caption: string;
  at?: string;
}

interface ServedBy {
  builds: number;
  closed: number;
  worktrees: number;
}

interface ItemTyping {
  type?: string;
  parent?: string | null;
  assignee?: string | null;
  stage?: string | null;
  fields?: Record<string, unknown>;
  blockedBy?: string[];
  blocked?: BlockedCause[];
  images?: ItemImage[];
  servedBy?: ServedBy;
}

interface WorkItem {
  id: string;
  title: string;
  status: string;
  thread: string;
  subject?: string;
  description?: string;
  ac?: Criterion[];
  rank?: number;
  updatedAt?: string;
  archivedAt?: string;
  deletedAt?: string;
  waitsOn?: WaitsOn | WaitsOn[];
  requestedBy?: RequestedBy | RequestedBy[];
}

export type { BlockedCause, BlockedKind, ItemImage, ItemTyping, ServedBy, WorkItem };
