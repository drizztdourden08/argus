/* @layer renderer-app @kind component */
import { AGENT_PREFIX } from '../behavior/look.constants';
import type { AssigneeMarkProps } from '../records.type';
import { AgentFace } from './AgentFace';
import { AssigneeFace } from './AssigneeFace';

const AssigneeMark = (props: AssigneeMarkProps) => (props.assignee.startsWith(AGENT_PREFIX) ? <AgentFace {...props} /> : <AssigneeFace {...props} />);

export { AssigneeMark };
