/* @layer renderer-app @kind logic */
import { LIVE_RUN_STATES } from './model.constants';
import type { RunEntry } from './model.type';

const isLiveRun = (run: RunEntry): boolean => LIVE_RUN_STATES.includes(run.status);

export { isLiveRun };
