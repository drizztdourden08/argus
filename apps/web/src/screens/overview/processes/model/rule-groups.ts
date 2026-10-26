/* @layer renderer-app @kind logic */
import type { ProcsMeta, RuleGroup } from '../../../../data/types';
import { asNumber } from '../../format';
import type { ProcItem } from './model.type';

const groupsFromRows = (procs: readonly ProcItem[]): RuleGroup[] => {
  const groups = new Map<string, { rule: string; count: number; memMB: number }>();
  for (const proc of procs) {
    if (!proc.rule) continue;
    const group = groups.get(proc.rule) ?? { rule: proc.rule, count: 0, memMB: 0 };
    group.count += 1;
    group.memMB += asNumber(proc.memMB) ?? 0;
    groups.set(proc.rule, group);
  }
  return [...groups.values()].sort((a, b) => b.memMB - a.memMB);
};

const ruleGroups = (meta: ProcsMeta | null, procs: readonly ProcItem[]): RuleGroup[] =>
  Array.isArray(meta?.byRule) ? meta.byRule.filter((group) => group.rule) : groupsFromRows(procs);

export { ruleGroups };
