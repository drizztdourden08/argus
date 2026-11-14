/* @layer renderer-app @kind logic */
import type { Option } from '../../../shared';
import type { AgentDoc } from '../kanban.type';
import { isLiveAgent } from './live-agent';
import { AGENT_PREFIX } from './model.constants';

const agentChoices = (agents: readonly AgentDoc[]): Option[] =>
  agents.filter(isLiveAgent).map((agent) => ({ value: `${AGENT_PREFIX}${agent.id}`, label: (agent.name?.trim() ?? '') || agent.id }));

export { agentChoices };
