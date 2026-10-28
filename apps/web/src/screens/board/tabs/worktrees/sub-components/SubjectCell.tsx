/* @layer renderer-app @kind component */
import { FoldText, Text } from '@drizztdourden08/tessera';
import { isLeftover } from '../model';
import type { CellProps } from '../worktrees.type';

const SubjectCell = (props: CellProps) => {
  const { row } = props;
  const text = isLeftover(row) ? (row.leftover?.topEntries ?? []).join(', ') : row.subject ?? '';
  if (!text) return null;
  return <Text as="div" variant="subtitle" title={text}><FoldText fill lines={1} overflow="clip">{text}</FoldText></Text>;
};

export { SubjectCell };
