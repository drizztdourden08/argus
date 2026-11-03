/* @layer renderer-app @kind component */
import { Button, Flex } from '@drizztdourden08/tessera';
import { OUTCOMES } from '../tests.constants';
import { outcomeTitle } from '../model';
import type { OutcomeButtonsProps } from '../tests-props.type';

const OutcomeButtons = (props: OutcomeButtonsProps) => {
  const { testId, active, busy, onChoose } = props;
  return (
    <Flex gap="xs" role="group" aria-label={`Run ${testId}`}>
      {OUTCOMES.map((outcome) => {
        const pressed = outcome.id !== 'pass' && active === outcome.id;
        return (
          <Button
            key={outcome.id}
            size="sm"
            variant={outcome.variant}
            active={pressed}
            disabled={busy}
            title={outcomeTitle(outcome.id, testId)}
            aria-pressed={outcome.id === 'pass' ? undefined : pressed}
            onClick={() => onChoose(outcome.id)}
          >
            {outcome.label}
          </Button>
        );
      })}
    </Flex>
  );
};

export { OutcomeButtons };
