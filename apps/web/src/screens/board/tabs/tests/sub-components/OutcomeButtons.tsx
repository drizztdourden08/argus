/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { OUTCOMES } from '../tests.constants';
import { outcomeTitle } from '../model';
import type { OutcomeButtonsProps } from '../tests-props.type';
import { TipButton } from './TipButton';

const OutcomeButtons = (props: OutcomeButtonsProps) => {
  const { testId, active, busy, onChoose } = props;
  return (
    <Flex gap="2xs" align="center" role="group" aria-label={`Run ${testId}`}>
      {OUTCOMES.map((outcome) => {
        const pressed = outcome.id !== 'pass' && active === outcome.id;
        return (
          <TipButton
            key={outcome.id}
            icon={outcome.icon}
            label={outcome.label}
            tip={outcomeTitle(outcome.id, testId)}
            variant={pressed ? outcome.variant : 'ghost'}
            active={pressed}
            disabled={busy}
            aria-pressed={outcome.id === 'pass' ? undefined : pressed}
            onClick={() => onChoose(outcome.id)}
          />
        );
      })}
    </Flex>
  );
};

export { OutcomeButtons };
