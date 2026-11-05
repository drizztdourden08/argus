/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { RecordLink } from '../../sub-components/RecordLink';
import { BoardList } from './BoardList';
import { BoardRow } from './BoardRow';
import type { BoardTabProps } from './board.type';
import { failingTests } from './model/failing-tests';

const BoardTestsTab = (props: BoardTabProps) => {
  const { data } = props;
  const { tests, testsPass } = data.record.counts;
  const failing = failingTests(data.tests);
  return (
    <Flex direction="column" gap="lg">
      <Text as="span">{`${testsPass} of ${tests} pass, ${failing.length} fail.`}</Text>
      <BoardList title="Failing" empty="No test fails." count={failing.length}>
        {failing.map((test) => (
          <BoardRow key={test.id}>
            <RecordLink id={test.id} board={data.record.key} />
            <Text as="span">{test.title}</Text>
          </BoardRow>
        ))}
      </BoardList>
    </Flex>
  );
};

export { BoardTestsTab };
