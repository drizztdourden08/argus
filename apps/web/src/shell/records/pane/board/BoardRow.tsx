/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import type { BoardRowProps } from './board.type';

const BoardRow = (props: BoardRowProps) => (
  <Flex gap="sm" align="center" role="listitem">
    {props.children}
  </Flex>
);

export { BoardRow };
