/* @layer renderer-app @kind logic */
import type { ProcItem } from './model.type';

const staleOf = (procs: readonly ProcItem[], rule: string): ProcItem[] =>
  procs.filter((proc) => proc.rule === rule && proc.status === 'stale' && proc.killable === true && proc.protected !== true);

export { staleOf };
