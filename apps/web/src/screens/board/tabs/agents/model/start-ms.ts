/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';

const startMs = (agent: Agent): number => Date.parse(agent.startedAt ?? agent.updatedAt ?? '');

export { startMs };
