/* @layer renderer-app @kind logic */
import { isLiveRun, runsOf } from '../../../../../data/runs';
import type { RunEntry } from '../../../../../data/runs';
import type { LaunchInfo } from './model.type';

const shownRun = (launch: LaunchInfo, runs: readonly RunEntry[]): RunEntry | null => {
  if (!launch.def) return null;
  const mine = runsOf(runs, launch.def.name);
  return mine.find(isLiveRun) ?? mine[0] ?? null;
};

export { shownRun };
