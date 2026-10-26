/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import type { CellProps } from '../worktrees.type';

const AheadCell = (props: CellProps) => {
  const { row } = props;
  if (!row.ahead && !row.behind) return null;
  return <Text numeric>{`${row.ahead ?? 0} ahead, ${row.behind ?? 0} behind`}</Text>;
};

export { AheadCell };
