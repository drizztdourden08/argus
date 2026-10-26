/* @layer renderer-app @kind logic */
import type { ThreadChoice } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';

const threadName = (threads: readonly WithId<ThreadChoice>[], id: string | null | undefined): string => {
  if (!id) return '';
  const thread = threads.find((candidate) => candidate.id === id);
  const session = thread?.session;
  if (session) return session;
  return id;
};

export { threadName };
