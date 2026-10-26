/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { useOutcome } from '../behavior/useOutcome';
import type { OutcomeControlsProps } from '../tests-props.type';
import { OutcomeButtons } from './OutcomeButtons';
import { ResultComposer } from './ResultComposer';

const OutcomeControls = (props: OutcomeControlsProps) => {
  const { test, boardKey } = props;
  const controls = useOutcome(boardKey, test.id);
  return (
    <Flex direction="column" gap="xs">
      <OutcomeButtons testId={test.id} active={controls.open} busy={controls.busy} onChoose={controls.choose} />
      {controls.open && <ResultComposer testId={test.id} outcome={controls.open} controls={controls} />}
    </Flex>
  );
};

export { OutcomeControls };
