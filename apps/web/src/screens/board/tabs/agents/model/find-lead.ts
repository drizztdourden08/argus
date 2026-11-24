/* @layer renderer-app @kind logic */
import type { ProjectThread } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';
import { isLeadRole } from '../../../../../data/content';

const isEnded = (thread: ProjectThread): boolean => thread.status === 'done' || thread.status === 'archived';

const findLead = (threads: readonly WithId<ProjectThread>[]): WithId<ProjectThread> | null => {
  const leads = threads.filter((thread) => isLeadRole(thread.role) && !thread.mergedInto && !thread.deletedAt);
  return leads.find((thread) => !isEnded(thread)) ?? leads[0] ?? null;
};

export { findLead };
