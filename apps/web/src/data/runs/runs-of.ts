/* @layer renderer-app @kind logic */
import { newestFirst } from './newest-first';
import type { RunEntry } from './runs.type';

const runsOf = (runs: readonly RunEntry[], name: string): RunEntry[] =>
  runs.filter((run) => run.name === name).sort(newestFirst);

export { runsOf };
