/* @layer renderer-app @kind logic */
import type { KanbanFilters } from './model.type';

const filtersActive = (filters: KanbanFilters): boolean =>
  Boolean(filters.query || filters.threads.length || filters.status || filters.subject || filters.needs || filters.conflict);

export { filtersActive };
