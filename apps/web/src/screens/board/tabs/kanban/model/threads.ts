/* @layer renderer-app @kind logic */
import type { ThreadChoice } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import { nonEmpty } from './non-empty';
import type { Option } from './model.type';

const isLiveThread = (thread: WithId<ThreadChoice>): boolean => !thread.mergedInto && !thread.deletedAt;

const threadChoices = (threads: readonly WithId<ThreadChoice>[], current?: string): Option[] => {
  const options = threads
    .filter((thread) => isLiveThread(thread) && (!thread.placeholder || thread.id === current))
    .map((thread) => ({ value: thread.id, label: `${nonEmpty(thread.session, thread.id)}${thread.status === 'archived' ? ' (archived)' : ''}` }));
  if (current && !options.some((option) => option.value === current)) options.push({ value: current, label: current });
  return options;
};

export { threadChoices };
