/* @layer renderer-app @kind logic */
import { isLeadRole } from '../../../../data/content';
import { ENDED_THREAD_STATUSES } from '../ThreadDrawer.constants';
import type { ThreadDoc, ThreadState } from '../ThreadDrawer.type';

const threadState = (thread: ThreadDoc): ThreadState => {
  const lead = isLeadRole(thread.role);
  const archived = thread.status === 'archived' || Boolean(thread.archivedAt);
  return { lead, archived, live: !lead && !archived && !ENDED_THREAD_STATUSES.includes(thread.status) };
};

export { threadState };
