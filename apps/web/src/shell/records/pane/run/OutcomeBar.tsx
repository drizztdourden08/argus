/* @layer renderer-app @kind component */
import { Button, Flex, Icon } from '@drizztdourden08/tessera';
import { RUN_OUTCOMES } from './run.constants';
import type { OutcomeBarProps } from './run.type';

const OutcomeBar = (props: OutcomeBarProps) => {
  const { disabled, first, last, onRecord, onStep } = props;
  return (
    <Flex gap="sm" align="center" justify="between" wrap>
      <Button size="sm" variant="ghost" icon={<Icon name="chevron-left" />} disabled={first} onClick={() => onStep(-1)}>Previous</Button>
      <Flex gap="xs" align="center" wrap>
        {RUN_OUTCOMES.map((outcome) => (
          <Button key={outcome.status} size="sm" variant={outcome.variant} disabled={disabled} onClick={() => onRecord(outcome.status)}>{outcome.label}</Button>
        ))}
      </Flex>
      <Button size="sm" variant="ghost" iconEnd={<Icon name="chevron-right" />} disabled={last} onClick={() => onStep(1)}>Next</Button>
    </Flex>
  );
};

export { OutcomeBar };
