/* @layer renderer-app @kind component */
import type { TreeCellProps } from '../review-props.type';
import { GhostCell } from './GhostCell';
import { PresentCell } from './PresentCell';

const TreeCell = (props: TreeCellProps) => (props.row.cells[props.account.key]?.present ? <PresentCell {...props} /> : <GhostCell {...props} />);

export { TreeCell };
