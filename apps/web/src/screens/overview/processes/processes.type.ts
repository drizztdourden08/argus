/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { ProcRule, ProcsMeta, RuleGroup } from '../../../data/types';
import type { KillAsk, ProcFilters, ProcItem, ProcRow } from './model';

interface ProcessFeed {
  ready: boolean;
  procs: ProcItem[];
  meta: ProcsMeta | null;
}

interface ProcessView {
  filters: ProcFilters;
  setFilters: (next: ProcFilters) => void;
  rows: ProcRow[];
  look: number;
  groups: RuleGroup[];
  rules: ProcRule[];
  ruleNames: string[];
}

interface KillFlow {
  ask: KillAsk | null;
  killing: boolean;
  askProcess: (id: string) => void;
  askRule: (rule: string) => void;
  cancel: () => void;
  killProcess: (id: string, tree: boolean) => void;
  killStale: (rule: string) => void;
}

interface ProcessSummaryProps {
  procs: readonly ProcItem[];
  meta: ProcsMeta | null;
}

interface ProcessFigureProps {
  before?: string;
  value: ReactNode;
  after?: string;
  title?: string;
}

interface SampleNoticesProps {
  meta: ProcsMeta | null;
}

interface RuleStripProps {
  groups: readonly RuleGroup[];
  rules: readonly ProcRule[];
  procs: readonly ProcItem[];
  rule: string;
  onPick: (rule: string) => void;
  kill: KillFlow;
}

interface RuleCardProps {
  group: RuleGroup;
  def: ProcRule | undefined;
  stale: readonly ProcItem[];
  selected: boolean;
  onPick: (rule: string) => void;
  kill: KillFlow;
}

interface ProcessFiltersProps {
  filters: ProcFilters;
  onChange: (next: ProcFilters) => void;
  total: number;
  look: number;
  rules: readonly string[];
}

interface ProcessTableProps {
  rows: readonly ProcRow[];
  kill: KillFlow;
}

interface KillCellProps {
  proc: ProcItem;
  kill: KillFlow;
}

interface ProcessCellProps {
  row: ProcRow;
  path: string;
  kill: KillFlow;
}

interface NameCellProps {
  name: string;
  pid: number | null;
}

interface StatusCellProps {
  proc: ProcItem;
}

interface KillRowConfirmProps {
  row: ProcRow;
  kill: KillFlow;
}

interface KillConfirmProps {
  question: string;
  busy: boolean;
  onKill: () => void;
  onKillTree?: () => void;
  onCancel: () => void;
}

interface RulesTableProps {
  rules: readonly ProcRule[];
}

export type {
  KillCellProps, KillConfirmProps, KillFlow, KillRowConfirmProps, NameCellProps, ProcessCellProps, ProcessFeed, ProcessFigureProps, ProcessFiltersProps, ProcessSummaryProps, ProcessTableProps, ProcessView, RuleCardProps,
  RuleStripProps, RulesTableProps, SampleNoticesProps, StatusCellProps,
};
