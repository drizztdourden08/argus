/* @layer renderer-app @kind types */
import type { ThreadChoice } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import type { LogDoc } from '../../../../../data/types';

type TrashKind = 'items' | 'tests' | 'actions' | 'entries' | 'agents';

interface TrashSource {
  id: string;
  title?: string;
  text?: string;
  name?: string;
  task?: string;
  kind?: string;
  thread?: string | null;
  deletedAt?: string | null;
  archivedAt?: string | null;
  deletedBy?: string | null;
  updatedBy?: string | null;
  deleteReason?: string | null;
}

type TrashInput = Record<TrashKind, readonly TrashSource[]>;

interface TrashRow {
  key: string;
  c: TrashKind;
  id: string;
  kind: string;
  text: string;
  thread: string;
  at: string;
  by: string;
  reason: string;
  archived: boolean;
}

type PurgeScope = 'thread' | 'keep' | 'status' | 'idle' | 'trash';

interface PurgeRequest {
  scope: PurgeScope;
  thread?: string;
}

interface PurgeDraft {
  scope: PurgeScope;
  thread: string;
  keep: readonly string[];
  status: readonly string[];
  idle: number;
  confirm: string;
}

type PurgeThread = WithId<ThreadChoice>;

interface PurgeDoc {
  id: string;
  thread?: string | null;
  item?: string | null;
  replyTo?: string | null;
  deletedAt?: string | null;
}

interface PurgeData {
  threads: readonly WithId<ThreadChoice>[];
  items: readonly (PurgeDoc & { ac?: readonly unknown[] })[];
  tests: readonly PurgeDoc[];
  actions: readonly PurgeDoc[];
  entries: readonly PurgeDoc[];
  log: readonly LogDoc[];
}

interface PurgeCounts {
  threads: number;
  items: number;
  ac: number;
  tests: number;
  entries: number;
  actions: number;
  logLines: number;
}

interface PurgeCriteria {
  threads?: string[];
  keep?: string[];
  status?: string[];
  idleDays?: number;
  trash?: boolean;
}

interface PurgeNotice {
  id: string;
  status: string;
  outcome: string;
}

export type {
  PurgeCounts, PurgeCriteria, PurgeData, PurgeDoc, PurgeDraft, PurgeNotice, PurgeRequest, PurgeScope, PurgeThread, TrashInput, TrashKind, TrashRow, TrashSource,
};
