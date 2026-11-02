/* @layer renderer-app @kind constants */
import type { DepState } from './deps.type';

const DEPS_COLLECTION = 'deps';

const WAITING_STATES: readonly DepState[] = ['opening', 'open'];

const LIVE_STATES: readonly DepState[] = ['opening', 'open', 'blocked'];

const LONG_WAIT_MS = 3 * 86_400_000;

export { DEPS_COLLECTION, LIVE_STATES, LONG_WAIT_MS, WAITING_STATES };
