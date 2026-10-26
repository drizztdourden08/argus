/* @layer renderer-app @kind logic */
import type { Entry } from '../../../../data/kanban';
import type { ThreadCounts } from '../ThreadDrawer.type';

const ofThread = (entry: Entry, thread: string): boolean => entry.thread === thread && !entry.deletedAt && !entry.archivedAt;

const threadCounts = (entries: readonly Entry[], thread: string): ThreadCounts => ({
  questions: entries.filter((entry) => ofThread(entry, thread) && entry.kind === 'question' && entry.status === 'open').length,
  toPickUp: entries.filter((entry) => ofThread(entry, thread) && entry.kind === 'decision' && entry.status === 'pending').length,
});

export { threadCounts };
