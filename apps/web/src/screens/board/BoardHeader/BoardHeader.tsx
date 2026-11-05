/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import type { BoardHeaderProps } from './BoardHeader.type';
import { BoardHeaderRow } from './sub-components/BoardHeaderRow';
import './BoardHeader.css';

const BoardHeader = (props: BoardHeaderProps) => (
  <Flex as="header" className="board-header" align="center">
    <BoardHeaderRow {...props} />
  </Flex>
);

export { BoardHeader };
