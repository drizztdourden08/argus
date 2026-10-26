/* @layer renderer-app @kind component */
import { Status } from '@drizztdourden08/tessera';
import type { AgentStatusProps } from '../AgentsTab.type';
import { isActive, isQuiet, statusLook } from '../model';

const AgentStatus = (props: AgentStatusProps) => {
  const { agent } = props;
  const look = statusLook(agent.status);
  const busy = isActive(agent) && !isQuiet(agent) && agent.status === 'working';
  return <Status tone={look.tone} pulse={busy} dot>{look.label}</Status>;
};

export { AgentStatus };
