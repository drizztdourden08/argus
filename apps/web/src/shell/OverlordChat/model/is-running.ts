/* @layer renderer-app @kind logic */
import type { OverlordState } from '../../../data/overlord';
import { RUNNING_STATES } from './model.constants';

const isRunning = (state: OverlordState | undefined): boolean => state !== undefined && RUNNING_STATES.includes(state);

export { isRunning };
