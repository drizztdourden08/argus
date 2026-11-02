/* @layer renderer-app @kind logic */
import type { RowTone } from '@drizztdourden08/tessera';
import { needsLook } from './needs-look';
import type { ProcRow } from './model.type';

const rowTone = (row: ProcRow): RowTone | undefined => (needsLook(row.item) ? 'warning' : undefined);

export { rowTone };
