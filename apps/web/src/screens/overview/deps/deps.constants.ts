/* @layer renderer-app @kind constants */
import type { DepFilter } from './model';

const DEP_FILTERS: { value: DepFilter; label: string }[] = [
  { value: 'open', label: 'Open' },
  { value: 'blocked', label: 'Blocked' },
  { value: 'resolved', label: 'Resolved' },
  { value: 'all', label: 'All' },
];

const TRAIL_MIN = 2;

const DEP_KIND = 'dependency';

const DEP_PANE_TAB = 'links';

const NO_DEPS: Readonly<Record<DepFilter, string>> = {
  open: 'No request is waiting on another project.',
  blocked: 'No request is blocked.',
  resolved: 'No request is resolved yet.',
  all: 'No dependency between projects yet.',
};

export { DEP_FILTERS, DEP_KIND, DEP_PANE_TAB, NO_DEPS, TRAIL_MIN };
