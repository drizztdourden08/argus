/* @layer renderer-app @kind logic */
import type { OverlordRule } from '../../../data/overlord';
import { minutesUntil } from '../../../format';

const nextRunText = (rule: OverlordRule, now: number): string => {
  if (!rule.enabled) return 'paused';
  const minutes = minutesUntil(rule.nextRunAt, now);
  if (minutes === null) return 'not scheduled';
  return minutes === 0 ? 'due now' : `in ${minutes} min`;
};

export { nextRunText };
