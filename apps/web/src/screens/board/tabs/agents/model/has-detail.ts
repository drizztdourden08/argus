/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import type { RowState } from '../AgentsTab.type';
import { hasMessage } from './has-message';

const hasDetail = (agent: Agent, id: string, state: RowState): boolean => hasMessage(agent, id, state) || state.editing === id;

export { hasDetail };
