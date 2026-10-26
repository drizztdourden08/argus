/* @layer renderer-app @kind logic */
import type { PurgeCounts } from './trash.type';

const purgeTotal = (counts: PurgeCounts): number =>
  counts.threads + counts.items + counts.ac + counts.tests + counts.entries + counts.actions + counts.logLines;

export { purgeTotal };
