/* @layer renderer-app @kind component */
import { Disclosure, Flex, List, SectionHeader } from '@drizztdourden08/tessera';
import { DONE_FOLD_AFTER } from '../model';
import type { TestDoc } from '../model';
import type { DoneListProps } from '../tests-props.type';
import { TestRow } from './TestRow';

const DoneList = (props: DoneListProps) => {
  const { tests, boardKey, onOpenTest } = props;
  const folded = tests.length > DONE_FOLD_AFTER;
  const rows = (list: TestDoc[]) => list.map((test) => <TestRow key={test.id} test={test} boardKey={boardKey} onOpenTest={onOpenTest} />);
  return (
    <Flex direction="column" gap="xs" role="group" aria-label="Done">
      <SectionHeader title="Done" count={tests.length} level={4} />
      <List>{rows(folded ? tests.slice(0, 1) : tests)}</List>
      {folded && (
        <Disclosure size="sm" summary={`Show all ${tests.length} done`}>
          <List>{rows(tests.slice(1))}</List>
        </Disclosure>
      )}
    </Flex>
  );
};

export { DoneList };
