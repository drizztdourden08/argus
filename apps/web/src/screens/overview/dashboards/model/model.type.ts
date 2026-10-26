/* @layer renderer-app @kind types */
import type { WithId } from '../../../../data/react';
import type { RegistryRow, SentEntry } from '../../../../data/types';

type LeadState = 'not started' | 'paused' | 'idle' | 'stale' | 'running';

interface Attention {
  needs: number;
  failing: number;
  blocked: number;
  decisions: number;
}

interface CardCountsView {
  agents: number;
  items: number;
  notPassing: number;
  failing: number;
}

interface SendOutcome {
  ok: boolean;
  message: string;
}

type SendKind = 'decision' | 'followup';

interface Totals {
  running: number;
  needs: number;
}

type RegistryItem = WithId<RegistryRow>;

type SentItem = WithId<SentEntry>;

type ChipTone = 'success' | 'warning' | 'danger' | 'neutral' | 'info';

interface RegistryChip {
  name: string;
  label: string;
  tone: ChipTone;
  pulse: boolean;
  tip: string;
  extra: string;
}

type ChipLook = Pick<RegistryChip, 'label' | 'tone' | 'pulse' | 'tip'>;

interface MasterLine {
  text: string;
  tone: 'muted' | 'danger' | 'success';
  running: boolean;
}

export type { Attention, CardCountsView, ChipLook, LeadState, MasterLine, RegistryChip, RegistryItem, SendKind, SendOutcome, SentItem, Totals };
