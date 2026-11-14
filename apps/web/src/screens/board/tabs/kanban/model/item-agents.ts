/* @layer renderer-app @kind logic */
import type { WithId } from '../../../../../data/react';
import type { AgentDoc } from '../kanban.type';
import { isLiveAgent } from './live-agent';

const itemAgents = (agents: readonly WithId<AgentDoc>[], itemId: string): WithId<AgentDoc>[] =>
  agents.filter((agent) => isLiveAgent(agent) && (agent.item === itemId || (agent.items ?? []).includes(itemId)));

export { itemAgents };
