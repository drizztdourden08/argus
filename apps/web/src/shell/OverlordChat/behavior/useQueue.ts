/* @layer renderer-app @kind logic */
import { APPROVALS_POLL_MS, decideApproval, readQueue, useAction } from '../../../data/overlord';
import type { ApprovalDecision } from '../../../data/overlord';
import { useRemoteRead } from '../../../data/remote';

const useQueue = () => {
  const read = useRemoteRead(readQueue, APPROVALS_POLL_MS);
  const action = useAction(read.reload);
  return {
    queue: read.data,
    problem: read.error,
    action,
    reload: read.reload,
    decide: (id: string, decision: ApprovalDecision) => action.run(() => decideApproval(id, decision)),
  };
};

export { useQueue };
