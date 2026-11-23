/* @layer renderer-app @kind constants */
import type { SegmentOption, SortDirection } from '@drizztdourden08/tessera';
import type { ReleasesFilters, ReleasesLayout, ReleasesOrder } from './releases.type';

const LOADING_TEXT = 'Loading the releases.';

const EMPTY_TEXT = 'No local releases yet.';

const EMPTY_HINT = 'A release made in local mode lands here with its notes, packages and app files, and waits for the owner\'s go to reach GitHub.';

const NO_MATCH_TEXT = 'No channel matches the filters.';

const RELEASES_TAB = 'releases';

const RELEASES_ORDERS: readonly { id: ReleasesOrder; label: string }[] = [
  { id: 'newest', label: 'Newest' },
  { id: 'channel', label: 'Channel' },
];

const LAYOUT_CHOICES: SegmentOption<ReleasesLayout>[] = [
  { value: 'normal', icon: 'rows-3', hint: { label: 'Normal', description: 'Each version with its details' } },
  { value: 'compact', icon: 'rows-4', hint: { label: 'Compact', description: 'Tighter cards' } },
];

const LAYOUT_IDS: ReleasesLayout[] = ['normal', 'compact'];

const RELEASES_DIRECTION: SortDirection = 'desc';

const RELEASES_FILTERS: ReleasesFilters = { query: '', channel: '' };

const RELEASES_SETUP = {
  groups: [] as string[],
  orders: RELEASES_ORDERS.map((choice): ReleasesOrder => choice.id),
  group: '',
  order: 'newest' as ReleasesOrder,
  direction: RELEASES_DIRECTION,
  filters: RELEASES_FILTERS,
};

export { EMPTY_HINT, EMPTY_TEXT, LAYOUT_CHOICES, LAYOUT_IDS, LOADING_TEXT, NO_MATCH_TEXT, RELEASES_ORDERS, RELEASES_SETUP, RELEASES_TAB };
