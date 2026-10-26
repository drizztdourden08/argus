/* @layer renderer-app @kind logic */
import type { ThreadChoice } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import { nonEmpty } from '../../../shared';
import type { FocusChoices } from './model.type';

const isFocusable = (thread: ThreadChoice): boolean =>
  !thread.mergedInto && thread.status !== 'done' && thread.status !== 'archived' && thread.role !== 'coordinator';

const focusChoices = (threads: readonly WithId<ThreadChoice>[], paused: ReadonlySet<string>): FocusChoices => {
  const live = threads.filter(isFocusable);
  return {
    options: live.map((thread) => ({ value: thread.id, label: `${nonEmpty(thread.session, thread.id)}${paused.has(thread.id) ? ' (paused)' : ''}` })),
    pausedCount: live.filter((thread) => paused.has(thread.id)).length,
  };
};

export { focusChoices };
