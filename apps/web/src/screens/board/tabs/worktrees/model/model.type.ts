/* @layer renderer-app @kind types */
import type { WithId } from '../../../../../data/react';
import type { Worktree } from '../../../../../data/worktrees';

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

interface RemoveLook {
  label: string;
  danger: boolean;
  tip: string;
}

interface WorktreeChip {
  id: string;
  text: string;
  title?: string;
}

export type {
  Counts, LeftoverFlag, Notice, Panel, PanelKind, RemoveLook, Seen, SummaryParts, WorktreeChip, WorktreeFilter, WorktreeRow,
};
