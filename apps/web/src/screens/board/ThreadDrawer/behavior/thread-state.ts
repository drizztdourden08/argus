/* @layer renderer-app @kind logic */
import { COORDINATOR_ROLE, ENDED_THREAD_STATUSES } from '../ThreadDrawer.constants';
import type { ThreadDoc, ThreadState } from '../ThreadDrawer.type';

const threadState = (thread: ThreadDoc): ThreadState => {
  const coordinator = thread.role === COORDINATOR_ROLE;
  const archived = thread.status === 'archived' || Boolean(thread.archivedAt);
  return { coordinator, archived, live: !coordinator && !archived && !ENDED_THREAD_STATUSES.includes(thread.status) };
};

export { threadState };
