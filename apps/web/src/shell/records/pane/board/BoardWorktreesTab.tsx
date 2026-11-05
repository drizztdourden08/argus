/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { BoardList } from './BoardList';
import { BoardRow } from './BoardRow';
import type { BoardTabProps } from './board.type';

const BoardWorktreesTab = (props: BoardTabProps) => {
  const { worktrees } = props.data;
  return (
    <BoardList title="Worktrees" empty="No worktrees." count={worktrees.length}>
      {worktrees.map((tree) => (
        <BoardRow key={tree.id}>
          <Text as="span" weight={600}>{tree.name ?? tree.branch ?? tree.path}</Text>
          <Text as="span" tone="muted">{tree.status}</Text>
        </BoardRow>
      ))}
    </BoardList>
  );
};

export { BoardWorktreesTab };
