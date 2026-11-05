/* @layer renderer-app @kind logic */
import type { TestFilters } from './model.type';

const filtersActive = (filters: TestFilters): boolean =>
  Boolean(filters.query || filters.threads.length > 0 || filters.runs.length > 0 || filters.status || filters.subject || filters.plan || filters.needs || filters.conflict);

export { filtersActive };
