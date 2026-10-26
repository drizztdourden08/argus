/* @layer renderer-app @kind component */
import { Button, Flex, ListItem, Text } from '@drizztdourden08/tessera';
import { rowKeys } from '../behavior/row-keys';
import { useOutcome } from '../behavior/useOutcome';
import { useRowMenu } from '../behavior/useRowMenu';
import { useTestsShared } from '../behavior/useTestsShared';
import { isToRun } from '../model';
import type { TestPartProps } from '../tests-props.type';
import { DoneSummary } from './DoneSummary';
import { OutcomeControls } from './OutcomeControls';
import { RerunButton } from './RerunButton';
import { TestChips } from './TestChips';
import { TestMenu } from './TestMenu';
import { WakeStatus } from './WakeStatus';

const TestRow = (props: TestPartProps) => {
  const { test } = props;
  const { boardKey, onOpen } = useTestsShared();
  const controls = useOutcome(boardKey, test.id);
  const menu = useRowMenu();
  const toRun = isToRun(test);
  return (
    <ListItem bare data-test={test.id} data-open-test={test.id} data-to-run={toRun ? '' : undefined} onKeyDown={rowKeys(toRun, controls, menu)}>
      <Flex direction="column" gap="xs">
        <Flex gap="sm" align="baseline" wrap>
          <Text mono tone="muted">{test.id}</Text>
          <Button size="sm" variant="ghost" onClick={() => onOpen(test.id)}>{test.title}</Button>
          <TestMenu test={test} menu={menu} />
        </Flex>
        <TestChips test={test} />
        {toRun ? (
          <OutcomeControls test={test} controls={controls} />
        ) : (
          <Flex gap="sm" align="center" wrap>
            <DoneSummary test={test} />
            <WakeStatus test={test} />
            <RerunButton test={test} boardKey={boardKey} />
          </Flex>
        )}
      </Flex>
    </ListItem>
  );
};

export { TestRow };
