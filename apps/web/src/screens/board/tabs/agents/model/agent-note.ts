/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import { isActive } from './is-active';

const agentNote = (agent: Agent): string => (isActive(agent) ? agent.note ?? '' : agent.result || agent.note || '');

export { agentNote };
