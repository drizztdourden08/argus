/* @layer renderer-app @kind component */
import { Button, Flex } from '@drizztdourden08/tessera';
import { OUTCOMES } from '../tests.constants';
import type { OutcomeButtonsProps } from '../tests-props.type';

const OutcomeButtons = (props: OutcomeButtonsProps) => {
  const { testId, active, busy, onChoose } = props;
  return (
    <Flex gap="xs" role="group" className="test-row__buttons" aria-label={`Run ${testId}`}>
      {OUTCOMES.map((outcome) => {
        const pressed = outcome.id !== 'pass' && active === outcome.id;
        return (
          <Button
            key={outcome.id}
            size="sm"
            variant={outcome.variant}
            active={pressed}
            disabled={busy}
            aria-label={`${outcome.label} ${testId}`}
            aria-pressed={outcome.id === 'pass' ? undefined : pressed}
            onClick={() => onChoose(outcome.id)}
          >
            {outcome.label.toLowerCase()}
          </Button>
        );
      })}
    </Flex>
  );
};

export { OutcomeButtons };
