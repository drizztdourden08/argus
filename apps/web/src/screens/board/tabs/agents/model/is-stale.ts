/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import { STALE_MS } from './model.constants';

const isStale = (agent: Agent, now: number): boolean =>
  agent.status === 'working' && Boolean(agent.updatedAt) && now - Date.parse(agent.updatedAt ?? '') > STALE_MS;

export { isStale };
