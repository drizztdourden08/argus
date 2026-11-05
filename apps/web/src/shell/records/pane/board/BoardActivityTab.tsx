/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { BoardList } from './BoardList';
import { BoardRow } from './BoardRow';
import type { BoardTabProps } from './board.type';
import { recentLines } from './model/recent-lines';

const BoardActivityTab = (props: BoardTabProps) => {
  const lines = recentLines(props.data);
  return (
    <BoardList title="Recent" empty={`Nothing yet on ${props.data.code}.`} count={lines.length}>
      {lines.map((line) => (
        <BoardRow key={line}>
          <Text as="span">{line}</Text>
        </BoardRow>
      ))}
    </BoardList>
  );
};

export { BoardActivityTab };
