/* @layer renderer-app @kind component */
import { Status } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../../../format';
import type { AgentStatusProps } from '../row-props.type';
import { agentNow, isActive, isQuiet, isStalled, statusLook } from '../model';
import { AgentCell } from './AgentCell';
import { AgentStatusTag } from './AgentStatusTag';

const statusTip = (props: AgentStatusProps): string => {
  const { agent, now } = props;
  const updated = agent.updatedAt ? `, updated ${formatAgo(agent.updatedAt, now)} ago` : '';
  const doing = agentNow(agent);
  return `${agent.status}${updated}${doing ? `\nNow: ${doing.text}` : ''}`;
};

const AgentStatus = (props: AgentStatusProps) => {
  const { agent } = props;
  const look = statusLook(agent.status);
  const busy = isActive(agent) && !isQuiet(agent) && agent.status === 'working';
  return (
    <AgentCell name="status" title={statusTip(props)} agentId={props.id}>
      <Status tone={isStalled(agent) ? 'warning' : look.tone} pulse={busy} dot dotSize="lg" halo aria-hidden>{null}</Status>
      <AgentStatusTag agent={agent} />
    </AgentCell>
  );
};

export { AgentStatus };
