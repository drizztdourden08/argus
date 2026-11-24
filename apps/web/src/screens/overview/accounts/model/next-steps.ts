/* @layer renderer-app @kind logic */
import type { NextRunPhase } from '../../../../data/types';
import { STEP_ASKED, STEP_DONE, STEP_LABELS, STEP_OF_PHASE, STEP_SWITCHED } from '../next.constants';
import type { NextSteps } from './next-steps.type';

const nextSteps = (phase: NextRunPhase = 'idle'): NextSteps => ({
  steps: [
    { id: STEP_ASKED, label: STEP_LABELS.asked },
    { id: STEP_DONE, label: phase === 'limit-confirmed' ? STEP_LABELS.limit : STEP_LABELS.done },
    { id: STEP_SWITCHED, label: STEP_LABELS.switched },
  ],
  currentId: STEP_OF_PHASE[phase],
  complete: phase === 'switched',
  tone: phase === 'failed' ? 'danger' : 'primary',
});

export { nextSteps };
