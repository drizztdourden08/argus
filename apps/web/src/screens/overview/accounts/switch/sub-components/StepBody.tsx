/* @layer renderer-app @kind component */
import type { ComponentType } from 'react';
import type { SwitchStage } from '../../../../../data/types';
import type { StepBodyProps } from '../switch.type';
import { CountdownBody } from './CountdownBody';
import { HandoverBody } from './HandoverBody';
import { SyncBody } from './SyncBody';
import { TextBody } from './TextBody';

const BODIES: Partial<Record<SwitchStage, ComponentType<StepBodyProps>>> = {
  countdown: CountdownBody,
  handover: HandoverBody,
  sync: SyncBody,
};

const StepBody = (props: StepBodyProps) => {
  const Body = BODIES[props.stage] ?? TextBody;
  return <Body {...props} />;
};

export { StepBody };
