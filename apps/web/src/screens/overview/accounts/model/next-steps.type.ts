/* @layer renderer-app @kind types */
import type { StepperProps } from '@drizztdourden08/tessera';
import type { NextRunPhase, NextWhen } from '../../../../data/types';

type NextSteps = Pick<StepperProps, 'steps' | 'currentId' | 'complete' | 'tone'>;

interface NextPair {
  from: string | null;
  to: string | null;
  when: NextWhen | undefined;
  phase: NextRunPhase | undefined;
}

export type { NextPair, NextSteps };
