/* @layer renderer-app @kind logic */
import { activityOf } from './activity-of';
import { presenceTip } from './presence-tip';
import { ringProgress } from './ring-progress';
import type { PresenceInput, PresenceLook } from '../OverlordPresence.type';

const presenceLook = (input: PresenceInput): PresenceLook => {
  const { status, now } = input;
  const activity = activityOf(status);
  return {
    activity,
    progress: ringProgress(status?.nextRuleAt, status?.nextRuleEveryMin, now),
    tip: presenceTip(activity, status, now),
  };
};

export { presenceLook };
