/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import { orText } from './or-text';

const agentName = (agent: Agent, id: string): string => orText(agent.name, id);

export { agentName };
