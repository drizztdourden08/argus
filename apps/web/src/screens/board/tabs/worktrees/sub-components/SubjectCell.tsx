/* @layer renderer-app @kind component */
import { isLeftover } from '../model';
import type { CellProps } from '../worktrees.type';
import { CellText } from './CellText';

const SubjectCell = (props: CellProps) => {
  const { row } = props;
  const text = isLeftover(row) ? (row.leftover?.topEntries ?? []).join(', ') : row.subject ?? '';
  if (!text) return null;
  return <CellText fill variant="subtitle" title={text}>{text}</CellText>;
};

export { SubjectCell };
