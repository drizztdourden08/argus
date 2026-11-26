/* @layer renderer-app @kind types */
import type { StepperProps } from '@drizztdourden08/tessera';
import type { NextRunPhase, NextWhen } from '../../../../data/types';
import type { SlotPair } from '../next.type';

type NextSteps = Pick<StepperProps, 'steps' | 'currentId' | 'complete' | 'tone'>;

interface NextPair {
  picked: SlotPair;
  auto: SlotPair;
  when: NextWhen | undefined;
  phase: NextRunPhase | undefined;
}

export type { NextPair, NextSteps };
