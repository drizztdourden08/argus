/* @layer renderer-app @kind logic */
import type { RowState } from '@drizztdourden08/tessera';
import { needsLook } from './needs-look';
import type { ProcRow } from './model.type';

const rowState = (row: ProcRow): RowState | undefined => (needsLook(row.item) ? 'edge' : undefined);

export { rowState };
