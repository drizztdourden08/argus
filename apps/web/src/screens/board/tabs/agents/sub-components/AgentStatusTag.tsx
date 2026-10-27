/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import type { AgentStatusTagProps } from '../AgentsTab.type';
import { statusLook, tagLook } from '../model';

const AgentStatusTag = (props: AgentStatusTagProps) => {
  const look = statusLook(props.agent.status);
  return <Tag {...tagLook(look.tone)}>{look.label}</Tag>;
};

export { AgentStatusTag };
