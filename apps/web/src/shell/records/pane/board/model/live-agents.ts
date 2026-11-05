/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';
import { FINISHED_AGENT_STATUSES } from '../board.constants';

const liveAgents = (agents: readonly WithId<Agent>[]): WithId<Agent>[] =>
  agents.filter((agent) => !agent.finishedAt && !agent.archivedAt && !agent.deletedAt && !FINISHED_AGENT_STATUSES.includes(agent.status));

export { liveAgents };
