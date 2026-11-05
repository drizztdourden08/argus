/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';
import type { Agent } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';
import type { AgentCompare } from './agent-compare.type';
import { agentName } from './agent-name';
import type { AgentOrder } from './agents-view.type';
import { startMs } from './start-ms';

const byStarted: AgentCompare = (a, b) => (startMs(a) || 0) - (startMs(b) || 0);

const byName: AgentCompare = (a, b) => agentName(a, a.id).localeCompare(agentName(b, b.id), undefined, { numeric: true });

const compareOf = (order: 'started' | 'name'): AgentCompare => (order === 'started' ? byStarted : byName);

const sortAgents = (agents: readonly WithId<Agent>[], order: AgentOrder, direction: SortDirection): WithId<Agent>[] => {
  if (order === 'updated') return direction === 'desc' ? [...agents] : [...agents].reverse();
  const compare = compareOf(order);
  const sign = direction === 'desc' ? -1 : 1;
  return [...agents].sort((a, b) => sign * compare(a, b));
};

export { sortAgents };
