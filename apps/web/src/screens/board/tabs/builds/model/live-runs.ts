/* @layer renderer-app @kind logic */
import { isLiveRun, newestFirst } from '../../../../../data/runs';
import type { RunEntry } from '../../../../../data/runs';

const liveRuns = (runs: readonly RunEntry[]): RunEntry[] => runs.filter(isLiveRun).sort(newestFirst);

export { liveRuns };
