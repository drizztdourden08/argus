/* @layer renderer-app @kind component */
import type { StepBodyProps } from '../switch.type';
import { BODIES } from './StepBody.constants';
import { TextBody } from './TextBody';

const StepBody = (props: StepBodyProps) => {
  const Body = BODIES[props.stage] ?? TextBody;
  return <Body {...props} />;
};

export { StepBody };
