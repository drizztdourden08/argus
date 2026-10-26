/* @layer renderer-app @kind logic */
import { staleOf } from './stale-of';
import type { KillAsk, ProcItem } from './model.type';

const askStillValid = (asked: KillAsk | null, procs: readonly ProcItem[]): KillAsk | null => {
  if (!asked) return null;
  if (asked.kind === 'proc') return procs.some((proc) => proc.id === asked.id) ? asked : null;
  return staleOf(procs, asked.rule).length > 0 ? asked : null;
};

export { askStillValid };
