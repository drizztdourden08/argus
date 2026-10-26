/* @layer renderer-app @kind constants */
import type { ListItemHeader, ListItemShape } from '@drizztdourden08/tessera';
import type { ActivityFilters } from './model';

const NOTHING_YET = 'Nothing to report yet.';
const NOTHING_MATCHES = 'Nothing matches these filters.';
const PAGE = 50;

const ACTIVITY_HEADER: ListItemHeader = {
  icon: 'Kind',
  main: 'Event',
  columns: [{ label: 'Level' }, { label: 'When', align: 'end' }],
};

const ACTIVITY_SHAPE: ListItemShape = { icon: true, columns: 2, action: false };

const NO_FILTERS: ActivityFilters = { level: 'all', kind: '' };

export { ACTIVITY_HEADER, ACTIVITY_SHAPE, NO_FILTERS, NOTHING_MATCHES, NOTHING_YET, PAGE };
