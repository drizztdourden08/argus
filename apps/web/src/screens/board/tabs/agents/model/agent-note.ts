/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import { isActive } from './is-active';
import { orText } from './or-text';

const agentNote = (agent: Agent): string => (isActive(agent) ? agent.note ?? '' : orText(agent.result, orText(agent.note, '')));

export { agentNote };
