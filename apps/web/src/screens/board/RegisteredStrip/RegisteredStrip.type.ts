/* @layer renderer-app @kind types */
import type { StatusTone } from '@drizztdourden08/tessera';
import type { ReactNode } from 'react';
import type { RegistryFault, RegistryRow } from '../../../data/types';

interface RegisteredStripProps {
  boardKey: string;
  folded?: ReactNode;
}

interface RegisteredLabelProps {
  rows: readonly RegistryRow[];
}

interface RegistryAgent {
  thread: string;
  status: string;
  updatedAt?: string;
  archivedAt?: string;
  deletedAt?: string;
  live?: { stalled?: boolean };
}

interface AgentLoad {
  working: number;
  stalled: number;
}

type ChipMark = 'dot' | 'warn' | 'spin';

interface ChipLook {
  tone: StatusTone;
  mark: ChipMark;
  word: string;
  faulted: boolean;
}

interface RegisteredChipProps {
  row: RegistryRow;
  load: AgentLoad;
}

interface AgentTextProps {
  load: AgentLoad;
}

interface ChipTipProps {
  row: RegistryRow;
}

interface FaultTipProps {
  fault: RegistryFault;
  main: boolean;
}

interface ChipMarkProps {
  look: ChipLook;
  name: string;
}

interface RegisteredView {
  rows: RegistryRow[];
  loads: ReadonlyMap<string, AgentLoad>;
}

export type {
  AgentLoad,
  AgentTextProps,
  ChipLook,
  ChipMarkProps,
  ChipTipProps,
  FaultTipProps,
  RegisteredChipProps,
  RegisteredLabelProps,
  RegisteredStripProps,
  RegisteredView,
  RegistryAgent,
};
