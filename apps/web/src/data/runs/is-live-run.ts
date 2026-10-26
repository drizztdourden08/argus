/* @layer renderer-app @kind logic */
import { LIVE_RUN_STATES } from './runs.constants';
import type { RunEntry } from './runs.type';

const isLiveRun = (run: RunEntry): boolean => LIVE_RUN_STATES.includes(run.status);

export { isLiveRun };
