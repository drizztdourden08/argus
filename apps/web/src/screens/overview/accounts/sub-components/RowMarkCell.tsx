/* @layer renderer-app @kind component */
import { TableCell } from '@drizztdourden08/tessera';
import type { ChipLook, RowMarkCellProps } from '../compare.type';
import { StatusChip } from './StatusChip';

const MARK_LOOK: Readonly<Record<'running' | 'done' | 'failed', ChipLook>> = {
  running: { label: 'running', tone: 'info' },
  done: { label: 'done', tone: 'success' },
  failed: { label: 'failed', tone: 'danger' },
};

const RowMarkCell = (props: RowMarkCellProps) => (
  <TableCell>{props.mark ? <StatusChip look={MARK_LOOK[props.mark]} /> : null}</TableCell>
);

export { RowMarkCell };
