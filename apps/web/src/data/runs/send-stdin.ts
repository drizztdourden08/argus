/* @layer renderer-app @kind logic */
import { apiRequest, scopePath } from '../api';

const sendStdin = (key: string, runId: string, text: string): Promise<{ ok: boolean; bytes: number }> =>
  apiRequest<{ ok: boolean; bytes: number }>(scopePath(key, 'runs', runId, 'stdin'), { method: 'POST', json: { text } });

export { sendStdin };
