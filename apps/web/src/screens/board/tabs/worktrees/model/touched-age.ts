/* @layer renderer-app @kind logic */
import { formatAgo } from '../../../../../format';
import { isLeftover } from './is-leftover';
import type { WorktreeRow } from './model.type';

const touchedAge = (row: WorktreeRow, now: number): string | undefined => {
  const at = isLeftover(row) ? row.leftover?.newestAt : row.lastTouchedAt;
  return at ? formatAgo(at, now) : undefined;
};

export { touchedAge };
