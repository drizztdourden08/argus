/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import { isActive } from './is-active';
import { orText } from './or-text';

const agentMessage = (agent: Agent): string => {
  const own = agent.live?.lastMessage;
  if (own) return own;
  return isActive(agent) ? '' : orText(agent.result, '');
};

export { agentMessage };
