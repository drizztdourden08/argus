/* @layer renderer-app @kind logic */
import type { OverlordActivity, OverlordStatus } from '../../../data/overlord';
import { minutesUntil } from '../../../format';
import { ACTIVITY_LABELS } from '../OverlordPresence.constants';

const presenceTip = (activity: OverlordActivity, status: OverlordStatus | null, now: number): string => {
  const label = ACTIVITY_LABELS[activity];
  const minutes = minutesUntil(status?.nextRuleAt, now);
  if (minutes === null || !status?.nextRuleTitle) return label;
  return `${label}. Next: ${status.nextRuleTitle} in ${minutes} min`;
};

export { presenceTip };
