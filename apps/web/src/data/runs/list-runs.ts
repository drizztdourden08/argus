/* @layer renderer-app @kind logic */
import { apiRequest, queryString, scopePath } from '../api';
import type { RunFilter, RunRow } from './runs.type';

const listRuns = async (key: string | null, filter: RunFilter = {}): Promise<RunRow[]> => {
  const path = key === null ? '/api/runs' : scopePath(key, 'runs');
  const result = await apiRequest<{ runs: RunRow[] }>(`${path}${queryString({ ...filter })}`);
  return result.runs;
};

export { listRuns };
