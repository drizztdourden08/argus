/* @layer renderer-app @kind logic */
import type { Entry, ThreadChoice } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import { byCreated } from './by-created';
import { isUnapplied } from './is-unapplied';
import { HUMAN, PAUSE_KINDS } from './model.constants';

const isPauseNote = (entry: Entry): boolean => entry.author === HUMAN && PAUSE_KINDS.includes(entry.kind) && Boolean(entry.replyTo) && isUnapplied(entry);

const pausedThreads = (threads: readonly WithId<ThreadChoice>[], entries: readonly WithId<Entry>[]): ReadonlySet<string> => {
  const paused = new Set(threads.filter((thread) => thread.status === 'paused').map((thread) => thread.id));
  for (const entry of entries.filter(isPauseNote).sort(byCreated)) {
    if (entry.kind === 'pause') paused.add(String(entry.replyTo));
    else paused.delete(String(entry.replyTo));
  }
  return paused;
};

export { pausedThreads };
