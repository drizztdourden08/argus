/* @layer renderer-app @kind constants */
import type { SortDirection } from '@drizztdourden08/tessera';
import type { ReleasesFilters, ReleasesOrder } from './releases.type';

const LOADING_TEXT = 'Loading the releases.';

const EMPTY_TEXT = 'No local releases yet.';

const EMPTY_HINT = 'A release made in local mode lands here with its notes, packages and app files, and waits for the owner\'s go to reach GitHub.';

const NO_MATCH_TEXT = 'No channel matches the filters.';

const RELEASES_TAB = 'releases';

const RELEASES_ORDERS: readonly { id: ReleasesOrder; label: string }[] = [
  { id: 'newest', label: 'Newest' },
  { id: 'channel', label: 'Channel' },
];

const RELEASES_DIRECTION: SortDirection = 'desc';

const RELEASES_FILTERS: ReleasesFilters = { query: '', channel: '' };

export { EMPTY_HINT, EMPTY_TEXT, LOADING_TEXT, NO_MATCH_TEXT, RELEASES_DIRECTION, RELEASES_FILTERS, RELEASES_ORDERS, RELEASES_TAB };
