/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import type { ActionHandlers } from '../../../../../shell/records';
import { copyText } from '../../../shared';
import type { AgentActions } from '../AgentsTab.type';
import { WORK_TAB } from '../AgentsTab.constants';
import type { OpenRecord } from '../row-props.type';

const canStop = (agent: Agent): boolean => agent.status === 'working' || agent.status === 'waiting';

const agentHandlers = (id: string, agent: Agent, actions: AgentActions, open: OpenRecord): ActionHandlers => ({
  open: () => open(id),
  stop: canStop(agent) ? () => actions.stop(id) : undefined,
  'set-item': () => open(id, WORK_TAB),
  archive: agent.archivedAt ? undefined : () => actions.soft('archive', id),
  unarchive: agent.archivedAt ? () => actions.soft('unarchive', id) : undefined,
  copy: () => copyText(id),
});

export { agentHandlers };
