/* @layer renderer-app @kind logic */
import type { Agent } from '../../../../../data/agents';
import { ACTIVE_STATUSES } from './model.constants';

const isActive = (agent: Agent): boolean => ACTIVE_STATUSES.includes(agent.status);

export { isActive };
