/* @layer renderer-app @kind types */
import type { Agent } from '../../../../data/agents';
import type { WithId } from '../../../../data/react';
import type { BatchJob, RemovedWorktree, WorktreeRequest, WorktreesMeta } from '../../../../data/worktrees';
import type { RecordHeadProps } from '../../../../shell';
import type { Counts, Notice, Panel, WorktreeChip, WorktreeGroup, WorktreeLayout, WorktreeRow } from './model';
import type { GroupThreads } from '../../item-groups';
import type { AskThread, BatchControls, PanelControls, Removal, TabPrefs } from './worktrees.type';

interface WorktreesTabProps {
  boardKey: string;
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
  layout: WorktreeLayout;
  compact: boolean;
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
  layout: WorktreeLayout;
  compact: boolean;
  meta: WorktreesMeta | null;
  controls: PanelControls;
  ask: AskThread;
  removal: Removal;
}

interface WorktreeCardProps {
  head: RecordHeadProps;
  title: string;
  chips: WorktreeChip[];
  time: string | number | undefined;
  job: WorktreeRow['job'];
  label: string;
  compact: boolean;
  onOpen: () => void;
}

interface WorktreeScannedProps {
  scannedAt?: string | number | null;
}

interface WorktreeCellsProps extends Omit<WorktreeRecordProps, 'row'> {
  rows: WorktreeRow[];
}

interface WorktreeBodyProps extends WorktreeGroupsProps {
  total: number;
  needOnly: boolean;
  hasRules: boolean;
  onShowAll: () => void;
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
  RefusedJobsProps, RemoveConfirmProps, RemovedRowProps, RemovedSectionProps, RequestFormProps, RequestRowProps, RequestsSectionProps, WorktreeBodyProps,
  WorktreeCardProps, WorktreeCellsProps, WorktreeGroupsProps, WorktreeRecordProps, WorktreeScannedProps, WorktreeToolbarProps, WorktreesTabProps,
};
