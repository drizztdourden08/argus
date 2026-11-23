/* @layer renderer-app @kind logic */
import { isNeeding } from './is-needing';
import type { WorktreeRow } from './model.type';

const filterRows = (rows: readonly WorktreeRow[], needOnly: boolean): WorktreeRow[] => (needOnly ? rows.filter(isNeeding) : [...rows]);

export { filterRows };
