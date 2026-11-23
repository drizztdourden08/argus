/* @layer renderer-app @kind logic */
import { isLeftover } from './is-leftover';
import type { WorktreeRow } from './model.type';

const touchedAt = (row: WorktreeRow): string | number | undefined => (isLeftover(row) ? row.leftover?.newestAt : row.lastTouchedAt) || undefined;

export { touchedAt };
