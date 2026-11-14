/* @layer renderer-app @kind component */
import { Avatar } from '@drizztdourden08/tessera';
import { OWNER_DEFAULTS } from '../../../owner';
import { useCollection } from '../../../data/react';
import { assigneeLook } from '../behavior/assignee-look';
import type { AgentName, AssigneeMarkProps } from '../records.type';

const AgentFace = (props: AssigneeMarkProps) => {
  const { assignee, boardKey, threads, size = 'xs' } = props;
  const agents = useCollection<AgentName>(boardKey, 'agents');
  const look = assigneeLook(assignee, boardKey, threads, OWNER_DEFAULTS, agents.docs);
  return <Avatar label={look.tip} initials={look.initials} variant="default" size={size} />;
};

export { AgentFace };
