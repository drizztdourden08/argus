/* @layer renderer-app @kind constants */
import type { DepColor, DepState } from './deps.type';

const DEPS_COLLECTION = 'deps';

const LIVE_STATES: readonly DepState[] = ['opening', 'open', 'blocked'];

const DEP_COLORS: Readonly<Record<DepState, DepColor>> = {
  opening: 'info',
  open: 'info',
  blocked: 'warning',
  done: 'success',
  dropped: 'neutral',
  gone: 'neutral',
  cancelled: 'neutral',
};

export { DEP_COLORS, DEPS_COLLECTION, LIVE_STATES };
