/* @layer renderer-app @kind logic */
import { apiRequest, queryString, scopePath } from '../api';

const readCheckLog = async (key: string, runId: string, step: number): Promise<string> =>
  (await apiRequest<string | null>(`${scopePath(key, 'checks', runId, 'log')}${queryString({ step })}`, { text: true })) ?? '';

export { readCheckLog };
