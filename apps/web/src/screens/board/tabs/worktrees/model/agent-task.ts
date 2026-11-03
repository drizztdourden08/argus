/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';
import { slashed } from './slashed';
import type { WorktreeRow } from './model.type';

const agentTask = (row: WorktreeRow, agents: readonly WithId<Agent>[]): string => {
  const owner = agents.find((agent) => row.agent && agent.id === row.agent) ?? agents.find((agent) => agent.worktree && slashed(agent.worktree) === slashed(row.path));
  return owner?.task?.trim() ?? '';
};

export { agentTask };
