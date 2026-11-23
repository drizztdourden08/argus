/* @layer renderer-app @kind types */
import type { WithId } from '../../../data/react';
import type { AccountsRun, ReviewCell, SyncStepDoc } from '../../../data/types';

type ChipTone = 'neutral' | 'success' | 'warning' | 'danger' | 'info';

interface ChipLook {
  label: string;
  tone: ChipTone;
}

type SyncStep = WithId<SyncStepDoc>;

interface RunFeed {
  run: AccountsRun | null;
  steps: SyncStep[];
}

interface StatusChipProps {
  look: ChipLook;
  title?: string;
}

interface CompareRow {
  id: string;
  kind: string;
  title?: string;
  cells: Record<string, ReviewCell | undefined>;
}

interface CompareDoc {
  accounts: { key: string; label?: string; color?: string }[];
  rows: CompareRow[];
}

export type { CompareDoc, RunFeed, StatusChipProps, SyncStep };
