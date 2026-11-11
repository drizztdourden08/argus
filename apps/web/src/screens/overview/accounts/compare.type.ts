/* @layer renderer-app @kind types */
import type { WithId } from '../../../data/react';
import type { AccountsRun, CompareAccount, CompareCell, CompareDoc, CompareRow, SyncStepDoc } from '../../../data/types';
import type { AccountActions } from './accounts.type';

type ChipTone = 'neutral' | 'success' | 'warning' | 'danger' | 'info';
type RowMark = 'running' | 'done' | 'failed';

interface ChipLook {
  label: string;
  tone: ChipTone;
}

type SyncStep = WithId<SyncStepDoc>;

interface CompareFeed {
  data: CompareDoc | null;
  loading: boolean;
  error: string | null;
  rescan: () => void;
}

interface RunFeed {
  run: AccountsRun | null;
  steps: SyncStep[];
}

interface PlanFlow {
  busy: boolean;
  note: string | null;
  run: () => void;
}

interface RowFilter {
  showSame: boolean;
  showArchived: boolean;
}

interface CompareViewProps {
  actions: AccountActions;
  onBack: () => void;
}

interface ApplyingViewProps {
  actions: AccountActions;
  onBack: () => void;
}

interface CompareToolbarProps {
  filter: RowFilter;
  busy: boolean;
  onFilter: (next: RowFilter) => void;
  onBack: () => void;
  onRescan: () => void;
  onPlan: () => void;
  onSync: () => void;
}

interface CompareSummaryProps {
  rows: readonly CompareRow[];
  same: number;
}

interface CompareTableProps {
  accounts: readonly CompareAccount[];
  rows: readonly CompareRow[];
  marks?: ReadonlyMap<string, RowMark>;
}

interface CompareCellViewProps {
  row: CompareRow;
  cell: CompareCell | undefined;
}

interface StatusChipProps {
  look: ChipLook;
  title?: string;
}

interface RowMarkCellProps {
  mark: RowMark | undefined;
}

interface StepKindTagsProps {
  steps: readonly SyncStep[];
}

interface StepLogProps {
  steps: readonly SyncStep[];
  accounts: readonly CompareAccount[];
}

interface RunFinishProps {
  phase: string;
  result: string | null | undefined;
  busy: boolean;
  onUndo: () => void;
  onBack: () => void;
}

export type {
  ApplyingViewProps, ChipLook, ChipTone, CompareCellViewProps, CompareFeed, CompareSummaryProps, CompareTableProps, CompareToolbarProps,
  CompareViewProps, PlanFlow, RowFilter, RowMark, RowMarkCellProps, RunFeed, RunFinishProps, StatusChipProps, StepKindTagsProps, StepLogProps, SyncStep,
};
