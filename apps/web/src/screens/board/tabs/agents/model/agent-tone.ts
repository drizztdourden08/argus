/* @layer renderer-app @kind logic */
import type { TrackRowTone } from '@drizztdourden08/tessera';
import type { Agent } from '../../../../../data/agents';
import { isStale } from './is-stale';
import { isStalled } from './is-stalled';

const agentTone = (agent: Agent, now: number): TrackRowTone | undefined => {
  if (isStalled(agent)) return 'warning';
  return isStale(agent, now) ? 'muted' : undefined;
};

export { agentTone };
