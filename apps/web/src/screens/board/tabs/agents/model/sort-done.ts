/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import type { WithId } from '../../../../../data/react';

const endedAt = (agent: Agent): string => agent.finishedAt ?? agent.updatedAt ?? '';

const sortDone = (agents: readonly WithId<Agent>[]): WithId<Agent>[] => [...agents].sort((a, b) => endedAt(b).localeCompare(endedAt(a)));

export { sortDone };
