/* @layer renderer-app @kind logic */
import type { CacheRow } from '../../../../../data/cache';

const pruneList = (rows: readonly CacheRow[]): CacheRow[] => rows.filter((row) => row.prunable > 0);

export { pruneList };
