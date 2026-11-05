/* @layer renderer-app @kind component */
import type { AssigneeMarkProps } from '../records.type';
import { AssigneeFace } from './AssigneeFace';

const AssigneeMark = (props: AssigneeMarkProps) => <AssigneeFace {...props} />;

export { AssigneeMark };
