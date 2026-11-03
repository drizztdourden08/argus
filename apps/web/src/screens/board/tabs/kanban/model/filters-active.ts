/* @layer renderer-app @kind logic */
import type { KanbanFilters } from './model.type';

const filtersActive = (filters: KanbanFilters): boolean =>
  Boolean(filters.query || filters.threads.length || filters.status || filters.types.length || filters.assignees.length || filters.needs || filters.conflict);

export { filtersActive };
