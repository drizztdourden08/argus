/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import type { AgentStatusTagProps } from '../row-props.type';
import { statusLook, tagLook } from '../model';

const AgentStatusTag = (props: AgentStatusTagProps) => {
  const look = statusLook(props.agent.status);
  return <Tag {...tagLook(look.tone)} className="agents-status-tag">{look.label}</Tag>;
};

export { AgentStatusTag };
