/* @layer renderer-app @kind logic */
import type { RunEntry } from '../../../../../data/runs';

const newestFirst = (a: RunEntry, b: RunEntry): number => b.queuedAt.localeCompare(a.queuedAt);

export { newestFirst };
