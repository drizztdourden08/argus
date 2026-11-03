/* @layer renderer-app @kind logic */
import type { PlanRun } from '../run.type';

const shownRun = (runs: readonly PlanRun[]): PlanRun | null => {
  const open = runs.find((run) => run.endedAt === null);
  if (open) return open;
  return runs.reduce<PlanRun | null>((latest, run) => (latest === null || run.n > latest.n ? run : latest), null);
};

export { shownRun };
