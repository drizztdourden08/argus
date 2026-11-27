/* @layer renderer-app @kind component */
import type { StepBodyProps } from '../switch.type';
import { CountdownLine } from './CountdownLine';
import { TextBody } from './TextBody';

const CountdownBody = (props: StepBodyProps) =>
  props.doc.phase === 'countdown' ? <CountdownLine doc={props.doc} /> : <TextBody {...props} />;

export { CountdownBody };
