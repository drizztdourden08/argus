/* @layer renderer-app @kind constants */
import type { DepFilter } from './model';

const DEP_FILTERS: { value: DepFilter; label: string }[] = [
  { value: 'open', label: 'Open' },
  { value: 'blocked', label: 'Blocked' },
  { value: 'resolved', label: 'Resolved' },
  { value: 'all', label: 'All' },
];

const NO_DEPS: Readonly<Record<DepFilter, string>> = {
  open: 'No request is waiting on another project.',
  blocked: 'No request is blocked.',
  resolved: 'No request is resolved yet.',
  all: 'No dependency between projects yet.',
};

const DEP_COLUMNS = ['Waiting', 'On', 'State', 'Waiting for', 'Last change'] as const;

export { DEP_COLUMNS, DEP_FILTERS, NO_DEPS };
