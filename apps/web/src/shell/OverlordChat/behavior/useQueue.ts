/* @layer renderer-app @kind logic */
import { APPROVALS_POLL_MS, readQueue, useAction } from '../../../data/overlord';
import { useRemoteRead } from '../../../data/remote';

const useQueue = () => {
  const read = useRemoteRead(readQueue, APPROVALS_POLL_MS);
  const action = useAction(read.reload);
  return {
    queue: read.data,
    problem: read.error,
    action,
    reload: read.reload,
  };
};

export { useQueue };
