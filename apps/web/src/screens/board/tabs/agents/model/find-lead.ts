/* @layer renderer-app @kind logic */
import type { ProjectThread } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';

const isEnded = (thread: ProjectThread): boolean => thread.status === 'done' || thread.status === 'archived';

const findLead = (threads: readonly WithId<ProjectThread>[]): WithId<ProjectThread> | null => {
  const leads = threads.filter((thread) => thread.role === 'coordinator' && !thread.mergedInto && !thread.deletedAt);
  return leads.find((thread) => !isEnded(thread)) ?? leads[0] ?? null;
};

export { findLead };
