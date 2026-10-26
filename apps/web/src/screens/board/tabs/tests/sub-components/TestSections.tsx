/* @layer renderer-app @kind component */
import { Flex, List, SectionHeader, Text } from '@drizztdourden08/tessera';
import { isToRun, sortDone, sortToRun } from '../model';
import type { TestSectionsProps } from '../tests-props.type';
import { DoneList } from './DoneList';
import { TestRow } from './TestRow';

const TestSections = (props: TestSectionsProps) => {
  const { tests, emptyText, boardKey, onOpenTest } = props;
  const toRun = sortToRun(tests.filter(isToRun));
  const done = sortDone(tests.filter((test) => !isToRun(test)));
  return (
    <Flex direction="column" gap="md">
      <Flex direction="column" gap="xs" role="group" aria-label="To run">
        <SectionHeader title="To run" count={toRun.length} level={4} />
        {toRun.length > 0 ? (
          <List>
            {toRun.map((test) => <TestRow key={test.id} test={test} boardKey={boardKey} onOpenTest={onOpenTest} />)}
          </List>
        ) : (
          <Text tone="muted">{tests.length > 0 ? 'Nothing left to run.' : emptyText}</Text>
        )}
      </Flex>
      {done.length > 0 && <DoneList tests={done} boardKey={boardKey} onOpenTest={onOpenTest} />}
    </Flex>
  );
};

export { TestSections };
