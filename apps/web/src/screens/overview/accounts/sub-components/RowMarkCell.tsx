/* @layer renderer-app @kind component */
import { TableCell } from '@drizztdourden08/tessera';
import type { RowMarkCellProps } from '../compare.type';
import { MARK_LOOK } from './RowMarkCell.constants';
import { StatusChip } from './StatusChip';

const RowMarkCell = (props: RowMarkCellProps) => (
  <TableCell>{props.mark ? <StatusChip look={MARK_LOOK[props.mark]} /> : null}</TableCell>
);

export { RowMarkCell };
