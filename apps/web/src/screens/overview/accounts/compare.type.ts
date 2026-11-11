/* @layer renderer-app @kind types */
import type { WithId } from '../../../data/react';
import type { AccountsRun, ReviewAccount, ReviewCell, ReviewDoc, ReviewRow, ReviewStatus, SyncStepDoc } from '../../../data/types';
import type { AccountActions } from './accounts.type';

type ChipTone = 'neutral' | 'success' | 'warning' | 'danger' | 'info';
type RowMark = 'running' | 'done' | 'failed';

interface ChipLook {
  label: string;
  tone: ChipTone;
}

type SyncStep = WithId<SyncStepDoc>;

interface RunFeed {
  run: AccountsRun | null;
  steps: SyncStep[];
}

interface ApplyingViewProps {
  actions: AccountActions;
  onBack: () => void;
}

interface ApplyingProgressProps {
  run: AccountsRun | null;
}

interface ApplyingTableProps {
  data: ReviewDoc | null;
  steps: readonly SyncStep[];
}

interface CellChipProps {
  status: ReviewStatus | undefined;
}

interface CompareTableProps {
  accounts: readonly ReviewAccount[];
  rows: readonly ReviewRow[];
  marks?: ReadonlyMap<string, RowMark>;
}

interface CompareCellViewProps {
  row: ReviewRow;
  cell: ReviewCell | undefined;
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
  accounts: readonly ReviewAccount[];
}

interface RunFinishProps {
  phase: string;
  result: string | null | undefined;
  busy: boolean;
  onUndo: () => void;
  onBack: () => void;
}

export type {
  ApplyingProgressProps, ApplyingTableProps, ApplyingViewProps, CellChipProps, ChipLook, CompareCellViewProps, CompareTableProps,
  RowMark, RowMarkCellProps, RunFeed, RunFinishProps, StatusChipProps, StepKindTagsProps, StepLogProps, SyncStep,
};
