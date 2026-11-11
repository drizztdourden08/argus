/* @layer renderer-app @kind component */
import { Tooltip } from '@drizztdourden08/tessera';
import type { RowTipProps } from '../review-tree-props.type';

const RowTip = (props: RowTipProps) => <Tooltip content={props.label}>{props.children}</Tooltip>;

export { RowTip };
