/* @layer renderer-app @kind logic */
import { apiRequest, queryString, scopePath } from '../api';
import type { RunLogPage, RunLogQuery } from './runs.type';

const fetchRunLogs = (key: string, runId: string, query: RunLogQuery = {}): Promise<RunLogPage> =>
  apiRequest<RunLogPage>(`${scopePath(key, 'runs', runId, 'logs')}${queryString({ ...query })}`);

export { fetchRunLogs };
