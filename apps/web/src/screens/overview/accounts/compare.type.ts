/* @layer renderer-app @kind types */
import type { WithId } from '../../../data/react';
import type { AccountsRun, SyncStepDoc } from '../../../data/types';

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

export type { RunFeed, StatusChipProps, SyncStep };
