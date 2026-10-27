/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import type { CellProps } from '../worktrees.type';

const AheadCell = (props: CellProps) => {
  const { row } = props;
  if (!row.ahead && !row.behind) return null;
  return <Text variant="caption" tone="dim" mono numeric>{`↑${row.ahead ?? 0} ↓${row.behind ?? 0}`}</Text>;
};

export { AheadCell };
