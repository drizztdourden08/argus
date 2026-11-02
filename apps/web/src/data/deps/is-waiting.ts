/* @layer renderer-app @kind logic */
import { WAITING_STATES } from './deps.constants';
import type { DepState } from './deps.type';

const isWaiting = (state: DepState): boolean => WAITING_STATES.includes(state);

export { isWaiting };
