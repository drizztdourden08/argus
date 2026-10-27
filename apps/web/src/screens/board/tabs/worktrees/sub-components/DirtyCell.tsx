/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import type { CellProps } from '../worktrees.type';

const DirtyCell = (props: CellProps) => {
  const { row } = props;
  return row.dirty ? <Text variant="caption" tone="dim" mono numeric>{row.dirty}</Text> : null;
};

export { DirtyCell };
