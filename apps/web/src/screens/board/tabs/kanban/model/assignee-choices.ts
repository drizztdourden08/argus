/* @layer renderer-app @kind logic */
import type { ItemRef, ThreadChoice } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import type { Option } from '../../../shared';
import { assigneeLook } from '../../../../../shell/records/behavior/assignee-look';
import type { AgentDoc } from '../kanban.type';
import { agentChoices } from './agent-choices';
import { assigneeOf } from './assignee-of';
import { OWNER } from './model.constants';

const assigneeChoices = (items: readonly ItemRef[], boardKey: string, threads: readonly WithId<ThreadChoice>[], agents: readonly AgentDoc[] = []): Option[] => {
  const live = agentChoices(agents);
  const found = new Set(items.filter((item) => !item.deletedAt).map(assigneeOf));
  found.delete(OWNER);
  for (const choice of live) found.delete(choice.value);
  const labelOf = (value: string): string => assigneeLook(value, boardKey, threads, { agents }).label;
  const others = [...found].sort().map((value) => ({ value, label: labelOf(value) }));
  return [{ value: OWNER, label: labelOf(OWNER) }, ...others, ...live];
};

export { assigneeChoices };
