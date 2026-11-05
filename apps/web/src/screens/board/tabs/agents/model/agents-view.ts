/* @layer renderer-app @kind logic */
import type { AgentViewSpec, ShownAgents } from './agents-view.type';
import { matchAgent } from './match-agent';
import { sortAgents } from './sort-agents';

const agentsView = (base: ShownAgents, spec: AgentViewSpec): ShownAgents => {
  const shape = (agents: ShownAgents['active']) => sortAgents(agents.filter((agent) => matchAgent(agent, spec.filters)), spec.order, spec.direction);
  return { active: shape(base.active), done: shape(base.done) };
};

export { agentsView };
