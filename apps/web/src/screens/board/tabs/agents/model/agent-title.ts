/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import { agentName } from './agent-name';

const agentTitle = (agent: Agent, id: string): string => {
  const name = agentName(agent, id);
  return agent.task ? `${name}: ${agent.task}` : name;
};

export { agentTitle };
