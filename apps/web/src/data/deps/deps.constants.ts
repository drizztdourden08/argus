/* @layer renderer-app @kind constants */
import type { DepColor, DepState } from './deps.type';

const DEPS_COLLECTION = 'deps';

const WAITING_STATES: readonly DepState[] = ['opening', 'open'];

const LIVE_STATES: readonly DepState[] = ['opening', 'open', 'blocked'];

const LONG_WAIT_MS = 3 * 86_400_000;

const DEP_COLORS: Readonly<Record<DepState, DepColor>> = {
  opening: 'info',
  open: 'info',
  blocked: 'warning',
  done: 'success',
  dropped: 'neutral',
  gone: 'neutral',
  cancelled: 'neutral',
};

export { DEP_COLORS, DEPS_COLLECTION, LIVE_STATES, LONG_WAIT_MS, WAITING_STATES };
