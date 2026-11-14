/* @layer renderer-app @kind logic */
import { FINISHED_AGENT_STATUSES } from '../../../../../shell/records/pane/board/board.constants';
import type { AgentDoc } from '../kanban.type';

const isLiveAgent = (agent: AgentDoc): boolean =>
  !agent.finishedAt && !agent.archivedAt && !agent.deletedAt && !FINISHED_AGENT_STATUSES.includes(agent.status ?? '');

export { isLiveAgent };
