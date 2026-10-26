/* @layer renderer-app @kind constants */
import type { ActivityFilters } from './model';

const NOTHING_YET = 'Nothing to report yet.';
const NOTHING_MATCHES = 'Nothing matches these filters.';
const PAGE = 50;
const ALL_KINDS = '*';
const NO_FILTERS: ActivityFilters = { level: 'all', kind: '' };

export { ALL_KINDS, NO_FILTERS, NOTHING_MATCHES, NOTHING_YET, PAGE };
