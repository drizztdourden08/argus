/* @layer renderer-app @kind logic */
import { apiRequest } from '../api';
import type { KillProcInput, KillResult } from './procs.type';

const killProc = (input: KillProcInput): Promise<KillResult> =>
  apiRequest<KillResult>('/api/procs/kill', { method: 'POST', json: input });

export { killProc };
