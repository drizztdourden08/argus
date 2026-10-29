/* @layer renderer-app @kind logic */
import { formatBytes } from '../../../../format';
import type { PruneResult } from '../../../../data/cache';
import type { Said } from './model.type';

const plural = (count: number, one: string, many: string): string => `${count} ${count === 1 ? one : many}`;

const pruneSaid = (result: PruneResult): Said => {
  const trouble = result.refused.length + result.failed.length;
  const done = `Deleted ${plural(result.deleted.length, 'cache entry', 'cache entries')} and ${plural(result.abandonedDeleted, 'temp folder', 'temp folders')}, ${formatBytes(result.freedBytes)} freed.`;
  if (trouble === 0) return { variant: 'success', message: done };
  return { variant: 'danger', message: `${done} ${plural(trouble, 'delete was', 'deletes were')} refused or failed, see the activity log.` };
};

export { pruneSaid };
