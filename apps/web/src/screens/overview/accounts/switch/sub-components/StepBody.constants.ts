/* @layer renderer-app @kind constants */
import type { ComponentType } from 'react';
import type { SwitchStage } from '../../../../../data/types';
import type { StepBodyProps } from '../switch.type';
import { CountdownBody } from './CountdownBody';
import { HandoverBody } from './HandoverBody';
import { SyncBody } from './SyncBody';

const BODIES: Partial<Record<SwitchStage, ComponentType<StepBodyProps>>> = {
  countdown: CountdownBody,
  handover: HandoverBody,
  sync: SyncBody,
};

export { BODIES };
