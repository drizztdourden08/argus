/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';

const isStalled = (agent: Agent): boolean => agent.status === 'working' && agent.live?.stalled === true;

export { isStalled };
