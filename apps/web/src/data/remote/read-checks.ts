/* @layer renderer-app @kind logic */
import { apiRequest, queryString, scopePath } from '../api';
import type { CheckQuery, CheckRun } from './remote.type';

const readChecks = async (key: string, query: CheckQuery = {}): Promise<CheckRun[]> =>
  (await apiRequest<{ runs: CheckRun[] }>(`${scopePath(key, 'checks')}${queryString({ ...query })}`)).runs;

export { readChecks };
