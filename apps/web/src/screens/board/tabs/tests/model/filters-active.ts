/* @layer renderer-app @kind logic */
import type { TestFilters } from './model.type';

const filtersActive = (filters: TestFilters): boolean =>
  Boolean(filters.query || filters.threads.length > 0 || filters.status || filters.subject || filters.plan || filters.needs || filters.conflict || filters.builds.length > 0 || filters.nobuild);

export { filtersActive };
