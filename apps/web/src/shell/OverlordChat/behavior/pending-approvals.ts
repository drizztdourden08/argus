/* @layer renderer-app @kind logic */
import type { OverlordApproval } from '../../../data/overlord';
import { toMillis } from '../../../format';

const pendingApprovals = (all: readonly OverlordApproval[], first: string | null): OverlordApproval[] => {
  const pending = all.filter((approval) => approval.status === 'pending').sort((a, b) => toMillis(a.at) - toMillis(b.at));
  const wanted = pending.find((approval) => approval.id === first);
  return wanted ? [wanted, ...pending.filter((approval) => approval !== wanted)] : pending;
};

export { pendingApprovals };
