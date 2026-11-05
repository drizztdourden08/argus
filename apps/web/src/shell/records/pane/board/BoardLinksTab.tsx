/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { RecordLink } from '../../sub-components/RecordLink';
import { BoardList } from './BoardList';
import { BoardRow } from './BoardRow';
import type { BoardTabProps } from './board.type';
import { depsOfBoard } from './model/deps-of-board';

const BoardLinksTab = (props: BoardTabProps) => {
  const { data } = props;
  const sides = depsOfBoard(data.deps, data.record.key);
  const groups = [
    { title: 'Waiting on other boards', rows: sides.outgoing, other: 'to' as const },
    { title: 'Asked of this board', rows: sides.incoming, other: 'from' as const },
  ];
  return (
    <Flex direction="column" gap="lg">
      {groups.map((group) => (
        <BoardList key={group.title} title={group.title} empty="No dependencies." count={group.rows.length}>
          {group.rows.map((dep) => (
            <BoardRow key={dep.id}>
              <RecordLink id={dep.id} />
              <Text as="span">{dep.title}</Text>
              <Text as="span" tone="muted">{`${dep[group.other].project}, ${dep.state}`}</Text>
            </BoardRow>
          ))}
        </BoardList>
      ))}
    </Flex>
  );
};

export { BoardLinksTab };
