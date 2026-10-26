/* @layer renderer-app @kind logic */
import type { RuleGroup } from '../../../../data/types';
import { asNumber, formatMemory } from '../../format';

const ruleTip = (group: RuleGroup, cap: number | null, over: boolean): string => {
  const base = `${group.rule}: ${asNumber(group.count) ?? 0} process(es), ${formatMemory(group.memMB)}`;
  if (cap === null) return base;
  return `${base}, group cap ${formatMemory(cap)}${over ? ' (over)' : ''}`;
};

export { ruleTip };
