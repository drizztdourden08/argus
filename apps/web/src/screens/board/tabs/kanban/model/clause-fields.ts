/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import type { ClauseField } from '../../../BoardTabsRow';
import { threadChoices } from '../../../shared';
import type { Threads } from '../kanban.type';
import { assigneeChoices } from './assignee-choices';
import { STATUS_OPTIONS } from './model.constants';
import type { KanbanFilters } from './model.type';
import { typeChoices } from './type-choices';

const kanbanFields = (boardKey: string, items: readonly ItemRef[], threads: Threads): ClauseField<KanbanFilters>[] => [
  { kind: 'multi', path: 'threads', label: 'Threads', options: threadChoices(threads), read: (filters) => filters.threads, write: (values) => ({ threads: values }) },
  { kind: 'multi', path: 'types', label: 'Type', options: typeChoices(), read: (filters) => filters.types, write: (values) => ({ types: values }) },
  { kind: 'multi', path: 'assignees', label: 'Assignee', options: assigneeChoices(items, boardKey, threads), read: (filters) => filters.assignees, write: (values) => ({ assignees: values }) },
  { kind: 'single', path: 'status', label: 'Status', options: STATUS_OPTIONS, read: (filters) => filters.status, write: (status) => ({ status }) },
  { kind: 'flag', path: 'needs', label: 'Needs you', read: (filters) => filters.needs, write: (needs) => ({ needs }) },
  { kind: 'flag', path: 'conflict', label: 'Conflict', read: (filters) => filters.conflict, write: (conflict) => ({ conflict }) },
  { kind: 'flag', path: 'archived', label: 'Archived', read: (filters) => filters.archived, write: (archived) => ({ archived }) },
];

export { kanbanFields };
