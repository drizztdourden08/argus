/* @layer renderer-app @kind logic */
import { EMPTY_ALL, EMPTY_MATCH, EMPTY_RUNNING } from './model.constants';
import type { AgentFilter } from './model.type';

const emptyMessage = (total: number, filter: AgentFilter): string => {
  if (total === 0) return EMPTY_ALL;
  return filter === 'running' ? EMPTY_RUNNING : EMPTY_MATCH;
};

export { emptyMessage };
