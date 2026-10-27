/* @layer renderer-app @kind logic */
import type { RunEntry } from './runs.type';

const newestFirst = (a: RunEntry, b: RunEntry): number =>
  b.queuedAt.localeCompare(a.queuedAt) || b.id.localeCompare(a.id, undefined, { numeric: true });

export { newestFirst };
