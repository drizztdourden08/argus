/* @layer renderer-app @kind types */
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
  selected: ProcItem | null;
  select: (id: string) => void;
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
  now: number;
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
  selectedId: string | null;
  onSelect: (id: string) => void;
}

interface ProcessDetailProps {
  proc: ProcItem;
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
  KillConfirmProps, KillFlow, ProcessDetailProps, ProcessFeed, ProcessFiltersProps, ProcessSummaryProps, ProcessTableProps, ProcessView, RuleCardProps,
  RuleStripProps, RulesTableProps, SampleNoticesProps,
};
