/* @layer renderer-app @kind logic */
import type { RunEntry } from './runs.type';

const runsOf = (runs: readonly RunEntry[], name: string): RunEntry[] =>
  runs.filter((run) => run.name === name).sort((a, b) => b.queuedAt.localeCompare(a.queuedAt));

export { runsOf };
