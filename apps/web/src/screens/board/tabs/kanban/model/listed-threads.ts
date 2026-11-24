/* @layer renderer-app @kind logic */
import type { ThreadChoice } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import { isLeadRole } from '../../../../../data/content';

const isHidden = (thread: ThreadChoice): boolean =>
  thread.status === 'done' || thread.status === 'archived' || Boolean(thread.archivedAt) || Boolean(thread.placeholder);

const listedThreads = (threads: readonly WithId<ThreadChoice>[]): string[] =>
  threads.filter((thread) => !thread.mergedInto && !thread.deletedAt && !isLeadRole(thread.role) && !isHidden(thread)).map((thread) => thread.id);

export { listedThreads };
