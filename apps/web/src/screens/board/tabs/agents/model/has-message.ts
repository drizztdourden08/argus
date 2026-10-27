/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import type { RowState } from '../AgentsTab.type';
import { agentMessage } from './agent-message';

const hasMessage = (agent: Agent, id: string, state: RowState): boolean => state.openMessages.has(id) && agentMessage(agent) !== '';

export { hasMessage };
