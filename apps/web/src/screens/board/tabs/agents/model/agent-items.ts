/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';

const agentItems = (agent: Agent): string[] => {
  if (agent.items && agent.items.length > 0) return agent.items;
  return agent.item ? [agent.item] : [];
};

export { agentItems };
