/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import { BUSY_ACTIVITIES } from './model.constants';

const isQuiet = (agent: Agent): boolean => {
  const live = agent.live;
  if (agent.status !== 'working' || !live) return false;
  return live.stalled === true || !BUSY_ACTIVITIES.includes(live.activity ?? '');
};

export { isQuiet };
