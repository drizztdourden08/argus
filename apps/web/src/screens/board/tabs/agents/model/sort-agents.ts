/* @layer renderer-app @kind logic */
import type { SortDirection } from '@drizztdourden08/tessera';
import type { Agent } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';
import { agentName } from './agent-name';
import type { AgentOrder } from './agents-view.type';
import { startMs } from './start-ms';

type Compare = (a: WithId<Agent>, b: WithId<Agent>) => number;

const byStarted: Compare = (a, b) => (startMs(a) || 0) - (startMs(b) || 0);

const byName: Compare = (a, b) => agentName(a, a.id).localeCompare(agentName(b, b.id), undefined, { numeric: true });

const COMPARES: Readonly<Record<'started' | 'name', Compare>> = { started: byStarted, name: byName };

const sortAgents = (agents: readonly WithId<Agent>[], order: AgentOrder, direction: SortDirection): WithId<Agent>[] => {
  if (order === 'updated') return direction === 'desc' ? [...agents] : [...agents].reverse();
  const compare = COMPARES[order];
  const sign = direction === 'desc' ? -1 : 1;
  return [...agents].sort((a, b) => sign * compare(a, b));
};

export { sortAgents };
