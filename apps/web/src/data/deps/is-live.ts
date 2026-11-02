/* @layer renderer-app @kind logic */
import { LIVE_STATES } from './deps.constants';
import type { DepState } from './deps.type';

const isLive = (state: DepState): boolean => LIVE_STATES.includes(state);

export { isLive };
