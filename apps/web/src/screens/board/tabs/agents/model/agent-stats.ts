/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import type { AgentStat } from './model.type';
import { statusLook } from './status-look';

const countOf = (active: readonly Agent[], status: string): number => active.filter((agent) => agent.status === status).length;

const statOf = (active: readonly Agent[], status: string): AgentStat => {
  const look = statusLook(status);
  return { id: status, label: look.label, count: countOf(active, status), tone: look.tone };
};

const agentStats = (active: readonly Agent[], finished: number): AgentStat[] => {
  const stopping = statOf(active, 'stop-requested');
  return [
    statOf(active, 'working'),
    statOf(active, 'waiting'),
    ...(stopping.count > 0 ? [stopping] : []),
    { id: 'finished', label: 'finished', count: finished, tone: 'neutral' },
  ];
};

export { agentStats };
