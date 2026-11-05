/* @layer renderer-app @kind logic */
import type { ProjectThread } from '../../../../../data/agents';
import type { Item } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import { itemCode } from '../../../../../shell';
import type { TabField } from '../../../BoardTabsRow';
import type { Option } from '../../../shared';
import { MODEL_CHOICES, STATUS_CHOICES } from './agents-view.constants';
import type { AgentFilterKey, AgentFilters } from './agents-view.type';
import { ITEM_LABEL_CHARS } from './model.constants';

const multi = (key: AgentFilterKey, label: string, options: readonly Option[]): TabField<AgentFilters> => ({
  kind: 'multi',
  path: key,
  label,
  options,
  read: (filters) => filters[key],
  write: (values) => ({ [key]: values }),
});

const itemChoices = (items: readonly WithId<Item>[]): Option[] =>
  items.map((item) => ({ value: item.id, label: `${itemCode(item)} ${String(item.title).slice(0, ITEM_LABEL_CHARS)}` }));

const agentFields = (threads: readonly WithId<ProjectThread>[], items: readonly WithId<Item>[]): TabField<AgentFilters>[] => [
  multi('thread', 'Thread', threads.map((thread) => ({ value: thread.id, label: thread.id }))),
  multi('status', 'Status', STATUS_CHOICES),
  multi('model', 'Model', MODEL_CHOICES),
  multi('item', 'Item', itemChoices(items)),
];

export { agentFields };
