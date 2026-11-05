/* @layer renderer-app @kind logic */
import type { BoardEntry, BoardsTotals } from '../boards-view.type';

const boardsTotals = (entries: readonly BoardEntry[]): BoardsTotals => {
  const active = entries.filter((entry) => !entry.record.archived);
  return {
    boards: active.length,
    running: active.filter((entry) => entry.record.lead === 'running').length,
    needs: active.filter((entry) => entry.record.started).reduce((sum, entry) => sum + entry.record.counts.needsYou, 0),
  };
};

export { boardsTotals };
