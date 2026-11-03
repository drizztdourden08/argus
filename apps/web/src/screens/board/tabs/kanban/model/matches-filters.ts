/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../data/kanban';
import { assigneeOf } from './assignee-of';
import { STATUS_OPTIONS } from './model.constants';
import { typeOf } from './type-of';
import type { KanbanFilters } from './model.type';

const matchesQuery = (item: ItemRef, query: string): boolean => {
  const needle = query.toLowerCase();
  return !needle || item.id.toLowerCase().includes(needle) || item.title.toLowerCase().includes(needle);
};

const matchesStatus = (item: ItemRef, status: string): boolean =>
  !status || !STATUS_OPTIONS.some((option) => option.value === status) || item.status === status;

const matchesPlace = (item: ItemRef, filters: KanbanFilters): boolean =>
  (filters.threads.length === 0 || filters.threads.includes(item.thread)) && (filters.assignees.length === 0 || filters.assignees.includes(assigneeOf(item)));

const matchesType = (item: ItemRef, types: readonly string[]): boolean => types.length === 0 || types.includes(typeOf(item));

const matchesAttention = (filters: KanbanFilters, needsYou: boolean, conflicted: boolean): boolean =>
  (!filters.needs || needsYou) && (!filters.conflict || conflicted);

const matchesFilters = (item: ItemRef, filters: KanbanFilters, needsYou: boolean, conflicted: boolean): boolean =>
  matchesQuery(item, filters.query) && matchesStatus(item, filters.status) && matchesPlace(item, filters) && matchesType(item, filters.types) && matchesAttention(filters, needsYou, conflicted);

export { matchesFilters };
