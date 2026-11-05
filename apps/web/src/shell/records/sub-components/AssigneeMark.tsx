/* @layer renderer-app @kind component */
import type { AssigneeMarkProps } from '../records.type';
import { AssigneeFace } from './AssigneeFace';
import { FrameMark } from './FrameMark';

const AssigneeMark = (props: AssigneeMarkProps) => (
  <FrameMark>
    <AssigneeFace {...props} />
  </FrameMark>
);

export { AssigneeMark };
