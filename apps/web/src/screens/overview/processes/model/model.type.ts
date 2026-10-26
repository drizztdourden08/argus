/* @layer renderer-app @kind types */
import type { WithId } from '../../../../data/react';
import type { ProcDoc } from '../../../../data/types';

type ProcItem = WithId<ProcDoc>;

interface ProcRow {
  id: string;
  rule: string;
  name: string;
  pid: number | null;
  project: string;
  memMB: number | null;
  cpuPct: number | null;
  ageMin: number | null;
  idleMin: number | null;
  status: string;
}

type ProcStatusFilter = 'all' | 'look';

interface ProcFilters {
  status: ProcStatusFilter;
  rule: string;
}

interface KillSentences {
  denied: string;
  gone: string;
  changed: string;
}

interface Said {
  variant: 'success' | 'danger';
  message: string;
}

type KillAsk = { kind: 'proc'; id: string } | { kind: 'rule'; rule: string };

export type { KillAsk, KillSentences, ProcFilters, ProcItem, ProcRow, Said };
