/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { RecordLink } from '../../sub-components/RecordLink';
import { BoardList } from './BoardList';
import { BoardRow } from './BoardRow';
import type { BoardTabProps } from './board.type';
import { openItems } from './model/open-items';

const BoardItemsTab = (props: BoardTabProps) => {
  const { data } = props;
  const groups = openItems(data.items);
  if (groups.length === 0) return <BoardList title="Open items" empty="No open items." count={0}>{null}</BoardList>;
  return (
    <Flex direction="column" gap="lg">
      {groups.map((group) => (
        <BoardList key={group.lane} title={group.label} empty="" count={group.items.length}>
          {group.items.map((item) => (
            <BoardRow key={item.id}>
              <RecordLink id={item.id} board={data.record.key} />
              <Text as="span">{item.title}</Text>
            </BoardRow>
          ))}
        </BoardList>
      ))}
    </Flex>
  );
};

export { BoardItemsTab };
