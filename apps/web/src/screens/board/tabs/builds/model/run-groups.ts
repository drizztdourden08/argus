/* @layer renderer-app @kind logic */
import { isLiveRun } from '../../../../../data/runs';
import type { RunEntry } from '../../../../../data/runs';
import { FINISHED_SHOWN } from '../builds.constants';
import type { LaunchDefDoc, RunGroups } from './model.type';
import { BUILD_RUN_NAME } from './run-groups.constants';

const newestFirst = (a: RunEntry, b: RunEntry): number => {
  const queued = b.queuedAt.localeCompare(a.queuedAt);
  return queued === 0 ? b.id.localeCompare(a.id, undefined, { numeric: true }) : queued;
};

const runGroups = (runs: readonly RunEntry[], defs: readonly LaunchDefDoc[]): RunGroups => {
  const names = new Set(defs.filter((def) => Boolean(def.launcher ?? def.bundle)).map((def) => def.name));
  const mine = runs.filter((run) => names.has(run.name) || BUILD_RUN_NAME.test(run.name)).sort(newestFirst);
  return { live: mine.filter(isLiveRun), finished: mine.filter((run) => !isLiveRun(run)).slice(0, FINISHED_SHOWN) };
};

export { runGroups };
