/* @layer renderer-app @kind types */
import type { BatchJob, WorktreeRequest, WorktreeRule, WorktreesMeta } from '../../../../data/worktrees';
import type { Counts, Notice, Panel, WorktreeFilter, WorktreeRow } from './model';
import type { AskThread, BatchControls, PanelControls, Removal, RowContext } from './worktrees.type';

interface WorktreesTabProps {
  boardKey: string;
}

interface WorktreeSummaryProps {
  counts: Counts;
  meta: WorktreesMeta | null;
}

interface WorktreeToolbarProps {
  filter: WorktreeFilter;
  onFilter: (filter: WorktreeFilter) => void;
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

interface RemoveButtonProps {
  row: WorktreeRow;
  context: RowContext;
}

interface WorktreeTableProps {
  rows: WorktreeRow[];
  meta: WorktreesMeta | null;
  controls: PanelControls;
  ask: AskThread;
  removal: Removal;
}

interface WorktreeBodyProps extends WorktreeTableProps {
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

interface RequestsSectionProps {
  requests: WorktreeRequest[];
}

interface RequestLineProps {
  request: WorktreeRequest;
}

export type {
  BatchConfirmProps, ConfirmFooterProps, DetailProps, LeftoverConfirmProps, LostTextProps, NoticeBannerProps,
  RefusedJobsProps, RemoveButtonProps, RemoveConfirmProps, RequestFormProps, RequestLineProps, RequestsSectionProps, RuleLineProps, RulesSectionProps, WorktreeBodyProps,
  WorktreeSummaryProps, WorktreeTableProps, WorktreeToolbarProps, WorktreesTabProps,
};
