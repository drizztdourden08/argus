/* @layer renderer-app @kind logic */
import type { ItemRef, ThreadChoice } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import type { Option } from '../../../shared';
import { assigneeLook } from '../../../../../shell/records/behavior/assignee-look';
import { assigneeOf } from './assignee-of';
import { OWNER } from './model.constants';

const assigneeChoices = (items: readonly ItemRef[], boardKey: string, threads: readonly WithId<ThreadChoice>[]): Option[] => {
  const found = new Set(items.filter((item) => !item.deletedAt).map(assigneeOf));
  found.delete(OWNER);
  const others = [...found].sort().map((value) => ({ value, label: assigneeLook(value, boardKey, threads).label }));
  return [{ value: OWNER, label: assigneeLook(OWNER, boardKey, threads).label }, ...others];
};

export { assigneeChoices };
