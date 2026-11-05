/* @layer renderer-app @kind types */
import type { Agent } from '../../../../data/agents';
import type { WithId } from '../../../../data/react';
import type { BatchJob, RemovedWorktree, WorktreeRequest, WorktreeRule, WorktreesMeta } from '../../../../data/worktrees';
import type { Counts, Notice, Panel, WorktreeFilter, WorktreeGroup, WorktreeRow } from './model';
import type { GroupThreads } from '../../item-groups';
import type { AskThread, BatchControls, PanelControls, Removal, TabPrefs } from './worktrees.type';

interface WorktreesTabProps {
  boardKey: string;
}

interface WorktreeSummaryProps {
  counts: Counts;
  meta: WorktreesMeta | null;
}

interface WorktreeToolbarProps {
  boardKey: string;
  rows: WorktreeRow[];
  prefs: TabPrefs;
  counts: Counts;
  meta: WorktreesMeta | null;
  batch: BatchControls;
}

interface BatchConfirmProps {
  meta: WorktreesMeta | null;
  batch: BatchControls;
}

interface NoticeBannerProps {
  notice: Notice;
  onDismiss: () => void;
}

interface RefusedJobsProps {
  jobs: BatchJob[];
  onDismiss: (ids: string[]) => void;
}

interface ConfirmFooterProps {
  stale: boolean;
  error: string | null;
  label: string;
  danger: boolean;
  disabled: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

interface DetailProps {
  row: WorktreeRow;
  meta: WorktreesMeta | null;
  controls: PanelControls;
  ask: AskThread;
  removal: Removal;
}

interface RequestFormProps {
  row: WorktreeRow;
  panel: Panel;
  meta: WorktreesMeta | null;
  controls: PanelControls;
  ask: AskThread;
}

interface RemoveConfirmProps {
  row: WorktreeRow;
  panel: Panel;
  controls: PanelControls;
  removal: Removal;
}

interface LostTextProps {
  row: WorktreeRow;
}

interface LeftoverConfirmProps extends RemoveConfirmProps {
  limit: number;
}

interface WorktreeGroupsProps {
  boardKey: string;
  rows: WorktreeRow[];
  group: WorktreeGroup;
  meta: WorktreesMeta | null;
  controls: PanelControls;
  ask: AskThread;
  removal: Removal;
}

interface WorktreeRecordProps {
  boardKey: string;
  row: WorktreeRow;
  threads: GroupThreads;
  agents: readonly WithId<Agent>[];
  now: number;
  meta: WorktreesMeta | null;
  controls: PanelControls;
  ask: AskThread;
  removal: Removal;
}

interface WorktreeBodyProps extends WorktreeGroupsProps {
  total: number;
  filter: WorktreeFilter;
  hasRules: boolean;
  onShowAll: () => void;
}

interface RulesSectionProps {
  rules: WorktreeRule[];
  meta: WorktreesMeta | null;
  open: boolean;
  onOpen: (open: boolean) => void;
}

interface RuleLineProps {
  rule: WorktreeRule;
  meta: WorktreesMeta | null;
}

interface CapToggleProps {
  total: number;
  limit: number;
  all: boolean;
  onToggle: () => void;
}

interface RequestsSectionProps {
  boardKey: string;
  requests: WorktreeRequest[];
}

interface RemovedSectionProps {
  boardKey: string;
  removed: RemovedWorktree[];
}

interface RequestRowProps {
  request: WorktreeRequest;
  boardKey: string;
  threads: GroupThreads;
  now: number;
}

interface RemovedRowProps {
  done: RemovedWorktree;
  boardKey: string;
  threads: GroupThreads;
  now: number;
}

export type {
  BatchConfirmProps, CapToggleProps, ConfirmFooterProps, DetailProps, LeftoverConfirmProps, LostTextProps, NoticeBannerProps,
  RefusedJobsProps, RemoveConfirmProps, RemovedRowProps, RemovedSectionProps, RequestFormProps, RequestRowProps, RequestsSectionProps, RuleLineProps, RulesSectionProps, WorktreeBodyProps,
  WorktreeGroupsProps, WorktreeRecordProps, WorktreeSummaryProps, WorktreeToolbarProps, WorktreesTabProps,
};
