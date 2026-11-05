/* @layer renderer-app @kind constants */
import { STAGE_LABELS } from '../builds.constants';
import type { BuildFilters, BuildGroup, BuildOrder } from './builds-view.type';

const BUILDS_TAB = 'builds';

const BUILD_FILTERS: BuildFilters = { query: '', item: [], kind: [], stage: [] };

const BUILD_GROUP_IDS: readonly BuildGroup[] = ['item', 'kind', 'none'];

const BUILD_ORDER_IDS: readonly BuildOrder[] = ['updated', 'name'];

const BUILD_GROUPS: readonly { id: BuildGroup; label: string }[] = [
  { id: 'item', label: 'Item' },
  { id: 'kind', label: 'Kind' },
  { id: 'none', label: 'None' },
];

const BUILD_ORDERS: readonly { id: BuildOrder; label: string }[] = [
  { id: 'updated', label: 'Updated' },
  { id: 'name', label: 'Name' },
];

const STAGE_CHOICES = Object.entries(STAGE_LABELS).map(([value, label]) => ({ value, label }));

const NO_KIND_LABEL = 'No kind';

const SEARCH_PLACEHOLDER = 'Search name, tag or branch';

export { BUILD_FILTERS, BUILD_GROUP_IDS, BUILD_GROUPS, BUILD_ORDER_IDS, BUILD_ORDERS, BUILDS_TAB, NO_KIND_LABEL, SEARCH_PLACEHOLDER, STAGE_CHOICES };
