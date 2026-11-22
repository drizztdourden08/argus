/* @layer renderer-app @kind logic */
import type { OverlordActivity, OverlordStatus } from '../../../data/overlord';
import { ACTIVITY_BY_STATE } from '../OverlordPresence.constants';

const activityOf = (status: OverlordStatus | null): OverlordActivity => {
  if (!status) return 'off';
  return status.activity ?? ACTIVITY_BY_STATE[status.status];
};

export { activityOf };
