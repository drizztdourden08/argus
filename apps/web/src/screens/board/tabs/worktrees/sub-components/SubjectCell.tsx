/* @layer renderer-app @kind component */
import { Text, Tooltip } from '@drizztdourden08/tessera';
import { isLeftover } from '../model';
import type { CellProps } from '../worktrees.type';

const SubjectCell = (props: CellProps) => {
  const { row } = props;
  const text = isLeftover(row) ? (row.leftover?.topEntries ?? []).join(', ') : row.subject ?? '';
  if (!text) return null;
  return <Tooltip content={text}><Text>{text}</Text></Tooltip>;
};

export { SubjectCell };
