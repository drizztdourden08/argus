/* @layer renderer-app @kind component */
import { Flex, Grid } from '@drizztdourden08/tessera';
import { WORKTREE_CARD_WIDTH } from '../worktrees.constants';
import type { WorktreeCellsProps } from '../worktrees-props.type';
import { WorktreeRecord } from './WorktreeRecord';

const WorktreeCells = (props: WorktreeCellsProps) => {
  const { rows, ...record } = props;
  const cells = rows.map((row) => <WorktreeRecord key={row.id} row={row} {...record} />);
  if (record.layout === 'list') return <Flex direction="column" gap="xs" role="list" aria-label="Worktrees">{cells}</Flex>;
  return <Grid minColWidth={WORKTREE_CARD_WIDTH} gap="sm" role="list" aria-label="Worktrees">{cells}</Grid>;
};

export { WorktreeCells };
