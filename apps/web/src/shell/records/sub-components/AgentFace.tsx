/* @layer renderer-app @kind component */
import { Avatar } from '@drizztdourden08/tessera';
import { useCollection } from '../../../data/react';
import { assigneeLook } from '../behavior/assignee-look';
import type { AgentName, AssigneeMarkProps } from '../records.type';

const AgentFace = (props: AssigneeMarkProps) => {
  const { assignee, boardKey, threads, size = 'xs' } = props;
  const agents = useCollection<AgentName>(boardKey, 'agents');
  const look = assigneeLook(assignee, boardKey, threads, { agents: agents.docs });
  return <Avatar label={look.tip} initials={look.initials ?? look.label.slice(0, 2).toUpperCase()} variant="default" size={size} />;
};

export { AgentFace };
