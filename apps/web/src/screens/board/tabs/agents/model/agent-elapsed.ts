/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import { formatDuration } from './format-duration';
import { isActive } from './is-active';
import { startMs } from './start-ms';

const agentElapsed = (agent: Agent, now: number): string => {
  const start = startMs(agent);
  const end = isActive(agent) ? now : Date.parse(agent.finishedAt ?? agent.updatedAt ?? '');
  return Number.isFinite(start) && Number.isFinite(end) ? formatDuration(end - start) : '-';
};

export { agentElapsed };
