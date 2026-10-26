/* @layer renderer-app @kind logic */
import type { ProjectThread } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';
import { CHAT_ORDER } from './model.constants';

const isHidden = (thread: ProjectThread): boolean =>
  thread.status === 'done' || thread.status === 'archived' || Boolean(thread.archivedAt) || Boolean(thread.placeholder);

const rankOf = (thread: ProjectThread): number => CHAT_ORDER[thread.status] ?? Object.keys(CHAT_ORDER).length;

const chatThreads = (threads: readonly WithId<ProjectThread>[]): WithId<ProjectThread>[] =>
  threads
    .filter((thread) => thread.role !== 'coordinator' && !thread.mergedInto && !thread.deletedAt && !isHidden(thread))
    .sort((a, b) => rankOf(a) - rankOf(b) || a.id.localeCompare(b.id));

export { chatThreads };
