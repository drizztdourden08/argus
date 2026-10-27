/* @layer renderer-app @kind logic */
import { isLiveRun } from '../../../../../data/runs';
import type { RunEntry } from '../../../../../data/runs';
import { RUN_CHIPS_SHOWN } from './model.constants';

const byLiveFirst = (a: RunEntry, b: RunEntry): number => Number(isLiveRun(b)) - Number(isLiveRun(a));

const runsByAgent = (runs: readonly RunEntry[]): Map<string, RunEntry[]> => {
  const grouped = new Map<string, RunEntry[]>();
  for (const run of runs) {
    if (run.agent) grouped.set(run.agent, [...(grouped.get(run.agent) ?? []), run]);
  }
  for (const [agent, mine] of grouped) grouped.set(agent, mine.sort(byLiveFirst).slice(0, RUN_CHIPS_SHOWN));
  return grouped;
};

export { runsByAgent };
