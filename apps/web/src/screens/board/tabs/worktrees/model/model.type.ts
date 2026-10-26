/* @layer renderer-app @kind types */
import type { StatusKey } from '@drizztdourden08/tessera';
import type { WithId } from '../../../../../data/react';
import type { Worktree } from '../../../../../data/worktrees';
import type { WORKTREE_STATUSES } from './model.constants';

type WorktreeRow = WithId<Worktree>;

type WorktreeFilter = 'all' | 'need' | 'left';

type PanelKind = 'delete' | 'check' | 'remove';

interface Seen {
  head: string | null;
  dirty: number;
  newestAt: string | null;
  discard: boolean;
  young: boolean;
}

interface Panel {
  id: string;
  kind: PanelKind;
  text: string;
  branch: boolean;
  error: string | null;
  seen: Seen;
}

interface Notice {
  text: string;
  backup: string | null;
  restore: string | null;
}

interface Counts {
  all: number;
  need: number;
  left: number;
  git: number;
}

interface SummaryParts {
  counts: string;
  need: string;
  gone: string | null;
  problems: string | null;
  many: string | null;
}

interface LeftoverFlag {
  text: string;
  tip: string;
}

type KnownStatus = StatusKey<typeof WORKTREE_STATUSES>;

interface RemoveLook {
  label: string;
  danger: boolean;
  tip: string;
}

interface TableRow {
  id: string;
  path: string;
  status: string;
  branch: string;
  touched: string;
  ahead: number;
  dirty: number;
  subject: string;
  request: string;
  actions: string;
  source: WorktreeRow;
}

export type {
  Counts, KnownStatus, LeftoverFlag, Notice, Panel, PanelKind, RemoveLook, Seen, SummaryParts, TableRow, WorktreeFilter, WorktreeRow,
};
