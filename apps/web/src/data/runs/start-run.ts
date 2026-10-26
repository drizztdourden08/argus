/* @layer renderer-app @kind logic */
import { apiRequest, scopePath } from '../api';
import type { RunRow, StartRunInput } from './runs.type';

const startRun = (key: string, input: StartRunInput): Promise<{ run: RunRow; already: boolean }> =>
  apiRequest<{ run: RunRow; already: boolean }>(scopePath(key, 'runs'), { method: 'POST', json: input });

export { startRun };
