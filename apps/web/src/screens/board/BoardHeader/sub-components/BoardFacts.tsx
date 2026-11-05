/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import type { BoardFactsProps } from '../BoardHeader.type';
import { BoardStamp } from './BoardStamp';
import { BoardSync } from './BoardSync';
import { BoardWorktrees } from './BoardWorktrees';
import './BoardFacts.css';

const BoardFacts = (props: BoardFactsProps) => {
  const { boardKey, status, updatedAt, className } = props;
  return (
    <Flex className={className ? `board-facts ${className}` : 'board-facts'} align="center" wrap role="group" aria-label="Board facts">
      <BoardStamp status={status} updatedAt={updatedAt} />
      <BoardWorktrees boardKey={boardKey} />
      <BoardSync boardKey={boardKey} />
    </Flex>
  );
};

export { BoardFacts };
