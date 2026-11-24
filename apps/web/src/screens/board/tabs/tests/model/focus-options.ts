/* @layer renderer-app @kind logic */
import type { ThreadChoice } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import { nonEmpty } from '../../../shared';
import type { FocusOption } from './model.type';
import { isLeadRole } from '../../../../../data/content';

const isFocusable = (thread: ThreadChoice): boolean =>
  !thread.mergedInto && thread.status !== 'done' && thread.status !== 'archived' && !isLeadRole(thread.role);

const focusOptions = (threads: readonly WithId<ThreadChoice>[], paused: ReadonlySet<string>): FocusOption[] =>
  threads.filter(isFocusable).map((thread) => ({
    value: thread.id,
    label: nonEmpty(thread.session, thread.id),
    paused: paused.has(thread.id),
  }));

export { focusOptions };
