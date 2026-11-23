/* @layer renderer-app @kind constants */
import { defineStatuses } from '@drizztdourden08/tessera';
import type { SegmentOption, SortDirection } from '@drizztdourden08/tessera';
import type { ChangesFilters, ChangesGroup, ChangesLayout, ChangesOrder } from './changes.type';

const CHECK_STATUSES = defineStatuses({
  queued: { label: 'Queued', tone: 'neutral' },
  running: { label: 'Running', tone: 'info', pulse: true },
  pass: { label: 'Passed', tone: 'success' },
  fail: { label: 'Failed', tone: 'danger' },
  cancelled: { label: 'Cancelled', tone: 'neutral' },
  'skipped-local': { label: 'Skipped locally', tone: 'warning' },
});

const PR_STATUSES = defineStatuses({
  open: { label: 'Open', tone: 'info' },
  merged: { label: 'Merged', tone: 'success' },
  closed: { label: 'Closed', tone: 'neutral' },
});

const CHANGE_KIND = 'change';

const SIGNAL_MAX = 3;

const LOADING_TEXT = 'Loading the pull requests.';

const EMPTY_TEXT = 'No pull requests yet.';

const PR_PANE_TAB = 'checks';

const CHECKS_READING = 'Reading the check runs.';

const CHECKS_NONE = 'No check run yet.';

const LOG_READING = 'Reading the log.';

const LOG_EMPTY = 'This step wrote no log.';

const CHANGES_TAB = 'changes';

const CHANGES_LAYOUT_FALLBACK: ChangesLayout = 'normal';

const LAYOUT_IDS: readonly ChangesLayout[] = ['normal', 'compact'];

const LAYOUT_CHOICES: SegmentOption<ChangesLayout>[] = [
  { value: 'normal', icon: 'rows-3', hint: { label: 'Normal', description: 'Two lines per pull request' } },
  { value: 'compact', icon: 'list', hint: { label: 'Compact', description: 'One line per pull request' } },
];

const FAILING_TIP = 'Open pull requests whose newest check run failed';

const NOT_ON_GITHUB_TIP = 'Pull requests not yet replayed to GitHub';

const CHANGES_GROUPS: readonly { id: ChangesGroup; label: string }[] = [
  { id: 'item', label: 'Item' },
  { id: 'status', label: 'Status' },
  { id: 'none', label: 'None' },
];

const CHANGES_ORDERS: readonly { id: ChangesOrder; label: string }[] = [
  { id: 'created', label: 'Created' },
  { id: 'merged', label: 'Merged' },
  { id: 'title', label: 'Title' },
];

const CHANGES_DIRECTION: SortDirection = 'desc';

const CHANGES_FILTERS: ChangesFilters = { query: '', status: '', item: '', author: '', base: '' };

const STATUS_OPTIONS = [
  { value: 'open', label: 'Open' },
  { value: 'merged', label: 'Merged' },
  { value: 'closed', label: 'Closed' },
];

const NO_MATCH_TEXT = 'No pull request matches the filters.';

const CHANGES_SETUP = {
  groups: CHANGES_GROUPS.map((choice): ChangesGroup => choice.id),
  orders: CHANGES_ORDERS.map((choice): ChangesOrder => choice.id),
  group: 'none' as ChangesGroup,
  order: 'created' as ChangesOrder,
  direction: CHANGES_DIRECTION,
  filters: CHANGES_FILTERS,
};

export { CHANGE_KIND, CHANGES_GROUPS, CHANGES_LAYOUT_FALLBACK, CHANGES_ORDERS, CHANGES_SETUP, CHANGES_TAB, FAILING_TIP, LAYOUT_CHOICES, LAYOUT_IDS, NO_MATCH_TEXT, NOT_ON_GITHUB_TIP, STATUS_OPTIONS, CHECK_STATUSES, CHECKS_NONE, CHECKS_READING, EMPTY_TEXT, LOADING_TEXT, LOG_EMPTY, LOG_READING, PR_PANE_TAB, PR_STATUSES, SIGNAL_MAX };
