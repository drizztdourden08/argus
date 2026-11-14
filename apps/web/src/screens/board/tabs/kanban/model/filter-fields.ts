/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { flagFields } from '../../../BoardTabsRow';
import type { TabField } from '../../../BoardTabsRow';
import { threadChoices } from '../../../shared';
import type { AgentDoc, Threads } from '../kanban.type';
import { assigneeChoices } from './assignee-choices';
import { STATUS_OPTIONS } from './model.constants';
import type { KanbanFilters } from './model.type';
import { typeChoices } from './type-choices';

const kanbanFields = (boardKey: string, items: readonly ItemRef[], threads: Threads, agents: readonly AgentDoc[]): TabField<KanbanFilters>[] => [
  { kind: 'multi', path: 'threads', label: 'Threads', options: threadChoices(threads), read: (filters) => filters.threads, write: (values) => ({ threads: values }) },
  { kind: 'multi', path: 'types', label: 'Type', options: typeChoices(), read: (filters) => filters.types, write: (values) => ({ types: values }) },
  { kind: 'multi', path: 'assignees', label: 'Assignee', options: assigneeChoices(items, boardKey, threads, agents), read: (filters) => filters.assignees, write: (values) => ({ assignees: values }) },
  { kind: 'single', path: 'status', label: 'Status', options: STATUS_OPTIONS, read: (filters) => filters.status, write: (status) => ({ status }) },
  ...flagFields<KanbanFilters>(),
];

export { kanbanFields };
