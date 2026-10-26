/* @layer renderer-app @kind logic */
import type { ProcRule, RuleGroup } from '../../../../data/types';
import { asNumber } from '../../format';

const groupCap = (group: RuleGroup, rule: ProcRule | undefined): number | null =>
  asNumber(rule?.maxGroupMemMB) ?? asNumber(group.maxGroupMemMB);

export { groupCap };
