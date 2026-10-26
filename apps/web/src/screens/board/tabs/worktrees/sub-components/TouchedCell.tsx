/* @layer renderer-app @kind component */
import { DateText } from '@drizztdourden08/tessera';
import { isLeftover } from '../model';
import type { CellProps } from '../worktrees.type';

const TouchedCell = (props: CellProps) => {
  const { row } = props;
  const leftover = isLeftover(row);
  const at = leftover ? row.leftover?.newestAt : row.lastTouchedAt;
  if (!at) return null;
  return <DateText value={at} format="elapsed" label={leftover ? 'last changed' : undefined} />;
};

export { TouchedCell };
