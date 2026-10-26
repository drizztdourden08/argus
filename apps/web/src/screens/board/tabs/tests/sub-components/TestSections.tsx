/* @layer renderer-app @kind component */
import { Flex, List, SectionHeader, Text } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import { doneTests, isToRun, sortToRun } from '../model';
import { KEY_HINT } from '../tests.constants';
import type { TestSectionsProps } from '../tests-props.type';
import { DoneList } from './DoneList';
import { TestRow } from './TestRow';

const TestSections = (props: TestSectionsProps) => {
  const { tests, emptyText } = props;
  const { order, showPassed } = useTestsShared();
  const toRun = sortToRun(tests.filter(isToRun), order);
  const done = doneTests(tests, showPassed);
  return (
    <Flex direction="column" gap="md">
      <Flex direction="column" gap="xs" role="group" aria-label="To run">
        <SectionHeader title="To run" count={toRun.length} level={4} />
        {toRun.length > 0 && <Text variant="caption" tone="muted">{KEY_HINT}</Text>}
        {toRun.length > 0 ? (
          <List>
            {toRun.map((test) => <TestRow key={test.id} test={test} />)}
          </List>
        ) : (
          <Text tone="muted">{tests.length > 0 ? 'Nothing left to run.' : emptyText}</Text>
        )}
      </Flex>
      {done.length > 0 && <DoneList tests={done} />}
    </Flex>
  );
};

export { TestSections };
