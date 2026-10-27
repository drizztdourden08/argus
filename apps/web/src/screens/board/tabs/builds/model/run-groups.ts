/* @layer renderer-app @kind logic */
import { finishedRuns, isLiveRun, newestFirst } from '../../../../../data/runs';
import type { RunEntry } from '../../../../../data/runs';
import type { LaunchDefDoc, RunGroups } from './model.type';
import { BUILD_RUN_NAME } from './run-groups.constants';

const runGroups = (runs: readonly RunEntry[], defs: readonly LaunchDefDoc[]): RunGroups => {
  const names = new Set(defs.filter((def) => Boolean(def.launcher ?? def.bundle)).map((def) => def.name));
  const mine = runs.filter((run) => names.has(run.name) || BUILD_RUN_NAME.test(run.name)).sort(newestFirst);
  return { live: mine.filter(isLiveRun), finished: finishedRuns(mine) };
};

export { runGroups };
