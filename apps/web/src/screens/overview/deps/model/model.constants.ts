/* @layer renderer-app @kind constants */
import type { DepState } from '../../../../data/deps';
import type { DepFilter } from './model.type';

const FILTER_STATES: Readonly<Record<DepFilter, readonly DepState[] | null>> = {
  open: ['opening', 'open'],
  blocked: ['blocked'],
  resolved: ['done', 'dropped', 'gone', 'cancelled'],
  all: null,
};

export { FILTER_STATES };
