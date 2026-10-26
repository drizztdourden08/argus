/* @layer renderer-app @kind logic */
import type { ProcsMeta } from '../../../../data/types';
import type { ProcItem } from './model.type';
import { ruleGroups } from './rule-groups';
import { rulesOf } from './rules-of';

const ruleNames = (meta: ProcsMeta | null, procs: readonly ProcItem[]): string[] => {
  const names = new Set<string>([
    ...rulesOf(meta).map((rule) => rule.name),
    ...ruleGroups(meta, procs).map((group) => group.rule),
    ...procs.map((proc) => proc.rule).filter((rule): rule is string => Boolean(rule)),
  ]);
  return [...names].sort();
};

export { ruleNames };
