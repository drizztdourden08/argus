/* @layer renderer-app @kind logic */
import { apiRequest, scopePath } from '../api';
import type { RunRow } from './runs.type';

const stopRun = (key: string, runId: string): Promise<{ run: RunRow }> =>
  apiRequest<{ run: RunRow }>(scopePath(key, 'runs', runId, 'stop'), { method: 'POST', json: {} });

export { stopRun };
