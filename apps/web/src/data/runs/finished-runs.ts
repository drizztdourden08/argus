/* @layer renderer-app @kind logic */
import { isLiveRun } from './is-live-run';
import { newestFirst } from './newest-first';
import { FINISHED_RUNS_SHOWN } from './runs.constants';
import type { RunEntry } from './runs.type';

const finishedRuns = (runs: readonly RunEntry[]): RunEntry[] =>
  runs.filter((run) => !isLiveRun(run)).sort(newestFirst).slice(0, FINISHED_RUNS_SHOWN);

export { finishedRuns };
