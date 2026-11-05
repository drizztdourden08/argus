/* @layer renderer-app @kind types */
import type { WorktreeRow } from './model.type';

type RowCompare = (a: WorktreeRow, b: WorktreeRow) => number;

export type { RowCompare };
