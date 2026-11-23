/* @layer renderer-app @kind logic */
import type { SwitchStepDoc } from '../../../../../data/types';
import { LEGACY_STAGE, LEGACY_STATE } from '../switch.constants';

const normalizeStep = (step: SwitchStepDoc): SwitchStepDoc => {
  if (step.stage !== undefined || step.id === undefined) return step;
  return {
    ...step,
    stage: LEGACY_STAGE[step.id],
    state: step.state ?? LEGACY_STATE[step.status ?? ''],
    text: step.text ?? step.label,
  };
};

export { normalizeStep };
