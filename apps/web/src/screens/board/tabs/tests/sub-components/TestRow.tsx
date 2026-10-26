/* @layer renderer-app @kind component */
import { Button, Flex, ListItem, Text } from '@drizztdourden08/tessera';
import { isToRun } from '../model';
import type { TestRowProps } from '../tests-props.type';
import { DoneSummary } from './DoneSummary';
import { OutcomeControls } from './OutcomeControls';
import { RerunButton } from './RerunButton';
import { TestChips } from './TestChips';

const TestRow = (props: TestRowProps) => {
  const { test, boardKey, onOpenTest } = props;
  const toRun = isToRun(test);
  return (
    <ListItem bare data-test={test.id}>
      <Flex direction="column" gap="xs">
        <Flex gap="sm" align="baseline" wrap>
          <Text mono tone="muted">{test.id}</Text>
          <Button size="sm" variant="ghost" onClick={() => onOpenTest(test.id)}>{test.title}</Button>
        </Flex>
        <TestChips test={test} />
        {toRun ? (
          <OutcomeControls test={test} boardKey={boardKey} />
        ) : (
          <Flex gap="sm" align="center" wrap>
            <DoneSummary test={test} />
            <RerunButton test={test} boardKey={boardKey} />
          </Flex>
        )}
      </Flex>
    </ListItem>
  );
};

export { TestRow };
