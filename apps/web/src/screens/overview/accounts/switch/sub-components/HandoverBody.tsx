/* @layer renderer-app @kind component */
import { detectDone, isActive } from '../model';
import type { StepBodyProps } from '../switch.type';
import { ThreadList } from './ThreadList';

const HandoverBody = (props: StepBodyProps) => (
  <ThreadList threads={props.doc.threads ?? []} checked={detectDone(props.doc)} skippable={isActive(props.doc)} />
);

export { HandoverBody };
